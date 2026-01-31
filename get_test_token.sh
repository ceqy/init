#!/bin/bash

# Configuration
HOST="localhost:50051"
TENANT_ID="00000000-0000-0000-0000-000000000001"
USERNAME="demo_ws"
PASSWORD="DemoPassword123!"
EMAIL="demo_ws@example.com"

echo "Using Tenant ID: $TENANT_ID"

# 1. Register (ignore error if user already exists)
echo "Attempting to register user '$USERNAME'..."
REGISTER_RESP=$(grpcurl -plaintext -d '{
  "tenant_id": "'$TENANT_ID'",
  "username": "'$USERNAME'",
  "email": "'$EMAIL'",
  "password": "'$PASSWORD'"
}' $HOST cuba.iam.user.UserService/Register 2>&1)

# Extract user_id from registration response
# Check if the response contains an error
if echo "$REGISTER_RESP" | grep -q "ERROR:"; then
    echo "User may already exist, skipping registration..."
else
    USER_ID=$(echo "$REGISTER_RESP" | jq -r '.userId // .user_id // empty')

    # 2. Activate user if registration was successful
    if [ -n "$USER_ID" ] && [ "$USER_ID" != "null" ]; then
        echo "New user registered with ID: $USER_ID"
        echo "Activating user via database (gRPC activation requires auth)..."
        docker exec postgres psql -U postgres -d cuba -c "UPDATE users SET status = 'Active' WHERE id = '$USER_ID';" > /dev/null 2>&1
        if [ $? -eq 0 ]; then
            echo "User activated successfully."
        else
            echo "Warning: Could not activate user automatically. You may need to activate manually."
        fi
    fi
fi

# 3. Login
echo "Logging in as '$USERNAME'..."
LOGIN_RESP=$(grpcurl -plaintext -d '{
  "tenant_id": "'$TENANT_ID'",
  "username": "'$USERNAME'",
  "password": "'$PASSWORD'",
  "ip_address": "127.0.0.1"
}' $HOST cuba.iam.auth.AuthService/Login 2>&1)

# Check if login failed due to inactive account
if echo "$LOGIN_RESP" | grep -q "User account is not active"; then
    echo "User account is not active. Attempting to activate via database..."
    docker exec postgres psql -U postgres -d cuba -c "UPDATE users SET status = 'Active' WHERE username = '$USERNAME';" > /dev/null 2>&1
    if [ $? -eq 0 ]; then
        echo "User activated. Retrying login..."
        LOGIN_RESP=$(grpcurl -plaintext -d '{
          "tenant_id": "'$TENANT_ID'",
          "username": "'$USERNAME'",
          "password": "'$PASSWORD'",
          "ip_address": "127.0.0.1"
        }' $HOST cuba.iam.auth.AuthService/Login 2>&1)
    fi
fi

# 4. Extract Token
ACCESS_TOKEN=$(echo $LOGIN_RESP | jq -r '.accessToken // .tokens.accessToken // .tokens.access_token // empty')

if [ -n "$ACCESS_TOKEN" ] && [ "$ACCESS_TOKEN" != "null" ]; then
    echo ""
    echo "✅ Login Successful!"
    echo "---------------------------------------------------"
    echo "JWT Token:"
    echo "$ACCESS_TOKEN"
    echo "---------------------------------------------------"
    echo "Copy the token above and paste it into the demo client."
else
    echo ""
    echo "❌ Login Failed."
    echo "Response:"
    echo "$LOGIN_RESP"
fi
