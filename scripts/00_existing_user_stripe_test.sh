#!/bin/bash

# 既存ユーザーでStripe動作確認

echo "=== Stripe Payment Integration Test (Existing User) ==="
echo

# 1. 既存ユーザーでログイン
echo "1. Logging in with existing user..."
LOGIN_RESPONSE=$(curl -s -X POST http://localhost:5000/auth/signin \
  -H "Content-Type: application/json" \
  -d '{
    "identifier": "stripe_test12@example.com",
    "password": "SecureP@ssw0rd!"
  }')

TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.tokens.access_token')
echo "Login response:"
echo "$LOGIN_RESPONSE" | jq '.success, .message' || echo "Login failed: $LOGIN_RESPONSE"

if [ "$TOKEN" == "null" ] || [ -z "$TOKEN" ]; then
  echo "Failed to get token. Full response:"
  echo "$LOGIN_RESPONSE" | jq '.'
  exit 1
fi

echo "Token obtained: ${TOKEN:0:20}..."
echo

# 2. 現在のサブスクリプション確認
echo "2. Checking current subscription..."
CURRENT_SUB=$(curl -s -X GET http://localhost:5000/subscriptions/current \
  -H "Authorization: Bearer $TOKEN")

echo "$CURRENT_SUB" | jq '.'
echo

# 3. 利用可能なプラン確認
echo "3. Getting available tiers..."
TIERS=$(curl -s -X GET http://localhost:5000/subscriptions/tiers \
  -H "Authorization: Bearer $TOKEN")

echo "$TIERS" | jq '.'
echo

# 4. チェックアウトセッション作成（Proプラン）
echo "4. Creating checkout session for Pro plan..."
CHECKOUT_RESPONSE=$(curl -s -X POST http://localhost:5000/payments/checkout \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "tier": "pro"
  }')

echo "$CHECKOUT_RESPONSE" | jq '.'
CHECKOUT_URL=$(echo "$CHECKOUT_RESPONSE" | jq -r '.data.checkout_url')
echo
echo "✅ Checkout URL: $CHECKOUT_URL"
echo

# 5. カスタマーポータルURL取得（Stripe顧客IDがない場合はエラーになる）
echo "5. Getting customer portal URL (may fail if no Stripe customer)..."
PORTAL_RESPONSE=$(curl -s -X POST http://localhost:5000/payments/portal \
  -H "Authorization: Bearer $TOKEN")

echo "$PORTAL_RESPONSE" | jq '.'
echo

echo "=== Test Complete ==="
echo
echo "🎯 Next steps:"
echo "1. Open the checkout URL in your browser: $CHECKOUT_URL"
echo "2. Complete the payment with test card:"
echo "   - Card number: 4242 4242 4242 4242"
echo "   - Expiry: Any future date (e.g., 12/34)"
echo "   - CVC: Any 3 digits (e.g., 123)"
echo "   - ZIP: Any 5 digits (e.g., 12345)"
echo "3. After payment, check the webhook logs in server.log"
echo
echo "📝 Useful commands:"
echo "   tail -f server.log | grep -i stripe"
echo "   tail -f server.log | grep -i webhook"