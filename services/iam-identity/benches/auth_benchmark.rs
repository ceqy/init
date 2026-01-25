//! 认证性能基准测试

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use iam_identity::auth::domain::services::*;
use iam_identity::shared::domain::value_objects::*;

fn password_hashing_benchmark(c: &mut Criterion) {
    c.bench_function("password_hash", |b| {
        b.iter(|| {
            PasswordService::hash_password(black_box("Test1234!"))
        })
    });
}

fn password_verification_benchmark(c: &mut Criterion) {
    let hashed = PasswordService::hash_password("Test1234!").unwrap();
    
    c.bench_function("password_verify", |b| {
        b.iter(|| {
            PasswordService::verify_password(black_box("Test1234!"), black_box(&hashed))
        })
    });
}

fn totp_generation_benchmark(c: &mut Criterion) {
    let service = TotpService::new("TestApp".to_string());
    
    c.bench_function("totp_generate_secret", |b| {
        b.iter(|| {
            service.generate_secret()
        })
    });
}

fn totp_verification_benchmark(c: &mut Criterion) {
    let service = TotpService::new("TestApp".to_string());
    let secret = service.generate_secret().unwrap();
    
    // 生成当前的 TOTP 码
    let totp = totp_rs::TOTP::new(
        totp_rs::Algorithm::SHA1,
        6,
        1,
        30,
        totp_rs::Secret::Encoded(secret.clone()).to_bytes().unwrap(),
    ).unwrap();
    let code = totp.generate_current().unwrap();
    
    c.bench_function("totp_verify", |b| {
        b.iter(|| {
            service.verify_code(black_box("testuser"), black_box(&secret), black_box(&code))
        })
    });
}

fn backup_code_generation_benchmark(c: &mut Criterion) {
    c.bench_function("backup_code_generate", |b| {
        b.iter(|| {
            BackupCodeService::generate_backup_codes(black_box(8))
        })
    });
}

criterion_group!(
    benches,
    password_hashing_benchmark,
    password_verification_benchmark,
    totp_generation_benchmark,
    totp_verification_benchmark,
    backup_code_generation_benchmark
);

criterion_main!(benches);
