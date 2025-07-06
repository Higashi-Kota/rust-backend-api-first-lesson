#!/bin/bash

# Stripe動作確認スクリプト

echo "=== Stripe Payment Integration Test ==="
echo

# 1. ユーザー登録
echo "1. Creating test user..."
SIGNUP_RESPONSE=$(curl -s -X POST http://localhost:5000/auth/signup \
  -H "Content-Type: application/json" \
  -d '{
    "email": "stripe_test12@example.com",
    "password": "SecureP@ssw0rd!",
    "username": "stripe_test12_user"
  }')

echo "Signup response:"
echo "$SIGNUP_RESPONSE" | jq '.'
echo

# 2. ログイン
echo "2. Logging in..."
LOGIN_RESPONSE=$(curl -s -X POST http://localhost:5000/auth/signin \
  -H "Content-Type: application/json" \
  -d '{
    "identifier": "stripe_test12@example.com",
    "password": "SecureP@ssw0rd!"
  }')

TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.tokens.access_token')
echo "Login response:"
echo "$LOGIN_RESPONSE" | jq '.success, .message'
echo "Token obtained: ${TOKEN:0:20}..."
echo

# 3. 現在のサブスクリプション確認
echo "3. Checking current subscription..."
CURRENT_SUB=$(curl -s -X GET http://localhost:5000/subscriptions/current \
  -H "Authorization: Bearer $TOKEN")

echo "$CURRENT_SUB" | jq '.'
echo

# 4. 利用可能なプラン確認
echo "4. Getting available tiers..."
TIERS=$(curl -s -X GET http://localhost:5000/subscriptions/tiers \
  -H "Authorization: Bearer $TOKEN")

echo "$TIERS" | jq '.'
echo

# 5. チェックアウトセッション作成（Proプラン）
echo "5. Creating checkout session for Pro plan..."
CHECKOUT_RESPONSE=$(curl -s -X POST http://localhost:5000/payments/checkout \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "tier": "pro"
  }')

echo "$CHECKOUT_RESPONSE" | jq '.'
CHECKOUT_URL=$(echo "$CHECKOUT_RESPONSE" | jq -r '.data.checkout_url')
echo
echo "Checkout URL: $CHECKOUT_URL"
echo

# 6. カスタマーポータルURL取得（Stripe顧客IDがない場合はエラーになる）
echo "6. Getting customer portal URL (may fail if no Stripe customer)..."
PORTAL_RESPONSE=$(curl -s -X POST http://localhost:5000/payments/portal \
  -H "Authorization: Bearer $TOKEN")

echo "$PORTAL_RESPONSE" | jq '.'
echo

echo "=== Test Complete ==="
echo
echo "Next steps:"
echo "1. Open the checkout URL in your browser: $CHECKOUT_URL"
echo "2. Complete the payment with test card: 4242 4242 4242 4242"
echo "3. Check the webhook logs in server.log"