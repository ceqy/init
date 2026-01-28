#!/bin/bash
TOKEN=$(jq -r '.access_token' token_response.json)
echo "Token: $TOKEN"
curl -v http://localhost:3000/api/audit/events \
  -H "Authorization: Bearer $TOKEN"
