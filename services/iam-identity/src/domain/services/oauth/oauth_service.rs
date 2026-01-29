use base64::{Engine as _, engine::general_purpose};
use cuba_common::{TenantId, UserId};
use cuba_errors::{AppError, AppResult};
use rand::Rng;
use sha2::{Digest, Sha256};
use tracing::debug;

use crate::domain::oauth::{AccessToken, AuthorizationCode, OAuthClientId, RefreshToken};

pub struct OAuthService {
    // 仓库现在通过 UnitOfWork 传递，这里可以保持为空，或者仅保留非仓库的依赖
}

impl OAuthService {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn create_authorization_code(
        &self,
        uow: &dyn crate::domain::unit_of_work::UnitOfWork,
        client_id: &OAuthClientId,
        user_id: &UserId,
        tenant_id: &TenantId,
        redirect_uri: String,
        scopes: Vec<String>,
        code_challenge: Option<String>,
        code_challenge_method: Option<String>,
    ) -> AppResult<String> {
        debug!("Creating authorization code for client: {}", client_id);

        let client_repo = uow.oauth_clients();
        let code_repo = uow.authorization_codes();

        let client = client_repo
            .find_by_id(client_id, tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("OAuth client not found"))?;

        if !client.redirect_uris.contains(&redirect_uri) {
            return Err(AppError::validation("Invalid redirect URI"));
        }

        let code = Self::generate_code();
        let authorization_code = AuthorizationCode::new(
            code.clone(),
            client_id.clone(),
            user_id.clone(),
            tenant_id.clone(),
            redirect_uri,
            scopes,
            code_challenge,
            code_challenge_method,
        );

        code_repo.save(&authorization_code).await?;

        Ok(code)
    }

    pub async fn exchange_code_for_token(
        &self,
        uow: &dyn crate::domain::unit_of_work::UnitOfWork,
        code: &str,
        client_id: &OAuthClientId,
        tenant_id: &TenantId,
        redirect_uri: &str,
        code_verifier: Option<&str>,
    ) -> AppResult<(String, String)> {
        debug!("Exchanging authorization code for token");

        let code_repo = uow.authorization_codes();
        let access_token_repo = uow.access_tokens();
        let refresh_token_repo = uow.refresh_tokens();

        let mut authorization_code = code_repo
            .find_by_code(code, tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("Authorization code not found"))?;

        if authorization_code.used {
            return Err(AppError::validation("Authorization code already used"));
        }

        if authorization_code.is_expired() {
            return Err(AppError::validation("Authorization code expired"));
        }

        if &authorization_code.client_id != client_id {
            return Err(AppError::validation("Invalid client"));
        }

        if authorization_code.redirect_uri != redirect_uri {
            return Err(AppError::validation("Invalid redirect URI"));
        }

        if let Some(challenge) = &authorization_code.code_challenge {
            let verifier =
                code_verifier.ok_or_else(|| AppError::validation("Code verifier required"))?;

            if !Self::verify_pkce(
                challenge,
                verifier,
                &authorization_code.code_challenge_method,
            ) {
                return Err(AppError::validation("Invalid code verifier"));
            }
        }

        authorization_code.mark_as_used();
        code_repo.update(&authorization_code).await?;

        let access_token = Self::generate_token();
        let refresh_token = Self::generate_token();

        let access_token_entity = AccessToken::new(
            access_token.clone(),
            client_id.clone(),
            Some(authorization_code.user_id.clone()),
            tenant_id.clone(),
            authorization_code.scopes.clone(),
            3600, // 1 hour
        );

        let refresh_token_entity = RefreshToken::new(
            refresh_token.clone(),
            access_token.clone(),
            client_id.clone(),
            authorization_code.user_id,
            tenant_id.clone(),
            authorization_code.scopes,
            2592000, // 30 days
        );

        access_token_repo.save(&access_token_entity).await?;
        refresh_token_repo.save(&refresh_token_entity).await?;

        Ok((access_token, refresh_token))
    }

    pub async fn refresh_access_token(
        &self,
        uow: &dyn crate::domain::unit_of_work::UnitOfWork,
        refresh_token: &str,
        client_id: &OAuthClientId,
        tenant_id: &TenantId,
    ) -> AppResult<(String, String)> {
        debug!("Refreshing access token");

        let access_token_repo = uow.access_tokens();
        let refresh_token_repo = uow.refresh_tokens();

        let mut refresh_token_entity = refresh_token_repo
            .find_by_token(refresh_token, tenant_id)
            .await?
            .ok_or_else(|| AppError::not_found("Refresh token not found"))?;

        if refresh_token_entity.revoked {
            return Err(AppError::validation("Refresh token revoked"));
        }

        if refresh_token_entity.is_expired() {
            return Err(AppError::validation("Refresh token expired"));
        }

        if &refresh_token_entity.client_id != client_id {
            return Err(AppError::validation("Invalid client"));
        }

        let old_access_token = access_token_repo
            .find_by_token(&refresh_token_entity.access_token, tenant_id)
            .await?;

        if let Some(mut old_token) = old_access_token {
            old_token.revoke();
            access_token_repo.update(&old_token).await?;
        }

        let new_access_token = Self::generate_token();
        let new_refresh_token = Self::generate_token();

        let access_token_entity = AccessToken::new(
            new_access_token.clone(),
            client_id.clone(),
            Some(refresh_token_entity.user_id.clone()),
            tenant_id.clone(),
            refresh_token_entity.scopes.clone(),
            3600, // 1 hour
        );

        let new_refresh_token_entity = RefreshToken::new(
            new_refresh_token.clone(),
            new_access_token.clone(),
            client_id.clone(),
            refresh_token_entity.user_id.clone(),
            tenant_id.clone(),
            refresh_token_entity.scopes.clone(),
            2592000, // 30 days
        );

        refresh_token_entity.revoke();
        refresh_token_repo.update(&refresh_token_entity).await?;

        access_token_repo.save(&access_token_entity).await?;
        refresh_token_repo.save(&new_refresh_token_entity).await?;

        Ok((new_access_token, new_refresh_token))
    }

    pub async fn revoke_token(
        &self,
        uow: &dyn crate::domain::unit_of_work::UnitOfWork,
        token: &str,
        tenant_id: &TenantId,
    ) -> AppResult<()> {
        debug!("Revoking token");

        let access_token_repo = uow.access_tokens();
        let refresh_token_repo = uow.refresh_tokens();

        if let Some(mut access_token) = access_token_repo.find_by_token(token, tenant_id).await? {
            access_token.revoke();
            access_token_repo.update(&access_token).await?;

            if let Some(mut refresh_token) = refresh_token_repo
                .find_by_access_token(token, tenant_id)
                .await?
            {
                refresh_token.revoke();
                refresh_token_repo.update(&refresh_token).await?;
            }
        } else if let Some(mut refresh_token) =
            refresh_token_repo.find_by_token(token, tenant_id).await?
        {
            refresh_token.revoke();
            refresh_token_repo.update(&refresh_token).await?;

            if let Some(mut access_token) = access_token_repo
                .find_by_token(&refresh_token.access_token, tenant_id)
                .await?
            {
                access_token.revoke();
                access_token_repo.update(&access_token).await?;
            }
        } else {
            return Err(AppError::not_found("Token not found"));
        }

        Ok(())
    }

    pub async fn introspect_token(
        &self,
        uow: &dyn crate::domain::unit_of_work::UnitOfWork,
        token: &str,
        tenant_id: &TenantId,
    ) -> AppResult<Option<AccessToken>> {
        debug!("Introspecting token");

        let access_token_repo = uow.access_tokens();
        let access_token = access_token_repo.find_by_token(token, tenant_id).await?;

        if let Some(token) = &access_token {
            if token.revoked || token.is_expired() {
                return Ok(None);
            }
        }

        Ok(access_token)
    }

    fn generate_code() -> String {
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.r#gen()).collect();
        general_purpose::URL_SAFE_NO_PAD.encode(&bytes)
    }

    fn generate_token() -> String {
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.r#gen()).collect();
        general_purpose::URL_SAFE_NO_PAD.encode(&bytes)
    }

    fn verify_pkce(challenge: &str, verifier: &str, method: &Option<String>) -> bool {
        match method.as_deref() {
            Some("S256") => {
                let mut hasher = Sha256::new();
                hasher.update(verifier.as_bytes());
                let result = hasher.finalize();
                let computed = general_purpose::URL_SAFE_NO_PAD.encode(&result);
                computed == challenge
            }
            Some("plain") | None => verifier == challenge,
            _ => false,
        }
    }
}
