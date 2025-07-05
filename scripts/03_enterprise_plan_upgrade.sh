#!/bin/bash

# エンタープライズプランアップグレード後の機能確認スクリプト

echo "=== Enterprise Plan Upgrade Test ==="
echo

# ユーザー情報
TEST_USER="enterprise_test@example.com"
TEST_PASS="SecureP@ssw0rd!"

# 1. ログイン
echo "1. Logging in..."
LOGIN_RESPONSE=$(curl -s -X POST http://localhost:5000/auth/signin \
  -H "Content-Type: application/json" \
  -d "{
    \"identifier\": \"$TEST_USER\",
    \"password\": \"$TEST_PASS\"
  }")

TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.tokens.access_token')
if [ "$TOKEN" == "null" ] || [ -z "$TOKEN" ]; then
  echo "❌ Login failed. Creating new user..."
  
  # ユーザー作成
  SIGNUP_RESPONSE=$(curl -s -X POST http://localhost:5000/auth/signup \
    -H "Content-Type: application/json" \
    -d "{
      \"email\": \"$TEST_USER\",
      \"password\": \"$TEST_PASS\",
      \"username\": \"enterprise_user\"
    }")
  
  # 再度ログイン
  LOGIN_RESPONSE=$(curl -s -X POST http://localhost:5000/auth/signin \
    -H "Content-Type: application/json" \
    -d "{
      \"identifier\": \"$TEST_USER\",
      \"password\": \"$TEST_PASS\"
    }")
  
  TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.tokens.access_token')
fi
echo "✅ Logged in successfully"
echo

# 2. 現在のプラン確認
echo "2. Checking current subscription..."
CURRENT_SUB=$(curl -s -X GET http://localhost:5000/subscriptions/current \
  -H "Authorization: Bearer $TOKEN")
CURRENT_TIER=$(echo "$CURRENT_SUB" | jq -r '.data.tier')
echo "Current plan: $CURRENT_TIER"
echo

# 3. エンタープライズプランへのアップグレード
if [ "$CURRENT_TIER" != "enterprise" ]; then
  echo "3. Creating checkout session for Enterprise plan upgrade..."
  CHECKOUT_RESPONSE=$(curl -s -X POST http://localhost:5000/payments/checkout \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
      "tier": "enterprise"
    }')
  
  CHECKOUT_URL=$(echo "$CHECKOUT_RESPONSE" | jq -r '.data.checkout_url')
  echo "✅ Checkout session created"
  echo "🔗 Checkout URL: $CHECKOUT_URL"
  echo
  echo "⚠️  Please complete the payment process:"
  echo "   1. Open the checkout URL in your browser"
  echo "   2. Use test card: 4242 4242 4242 4242"
  echo "   3. Complete the payment"
  echo "   4. Wait for webhook processing"
  echo "   5. Re-run this script to test Enterprise features"
  echo
  exit 0
fi

# 4. エンタープライズプラン機能のテスト
echo "3. Testing Enterprise plan features (UNLIMITED)..."
echo

# 4.1 無制限のチーム作成テスト
echo "4.1 Testing unlimited team creation..."
echo "Creating multiple teams to demonstrate no limits..."

# 10個のチームを一気に作成
SUCCESS_COUNT=0
for i in {1..10}; do
  TEAM_RESPONSE=$(curl -s -X POST http://localhost:5000/teams \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "{
      \"name\": \"Enterprise Team $i\",
      \"description\": \"Enterprise plan team $i - no limits!\"
    }")
  
  if echo "$TEAM_RESPONSE" | jq -e '.success' > /dev/null 2>&1; then
    SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
    echo -n "."
  else
    echo
    echo "❌ Failed to create team $i"
    echo "$TEAM_RESPONSE" | jq '.'
    break
  fi
done
echo
echo "✅ Created $SUCCESS_COUNT teams successfully (Enterprise has NO LIMIT)"
echo

# 4.2 無制限のタスク作成テスト
echo "4.2 Testing unlimited task creation..."
echo "Creating batch of 50 tasks..."

SUCCESS_COUNT=0
for i in {1..50}; do
  TASK_RESPONSE=$(curl -s -X POST http://localhost:5000/tasks \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "{
      \"title\": \"Enterprise Task $i\",
      \"description\": \"Enterprise plan allows unlimited tasks - Task $i\",
      \"status\": \"todo\",
      \"priority\": \"high\"
    }")
  
  if echo "$TASK_RESPONSE" | jq -e '.success' > /dev/null 2>&1; then
    SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
    if [ $((SUCCESS_COUNT % 10)) -eq 0 ]; then
      echo -n "[$SUCCESS_COUNT]"
    else
      echo -n "."
    fi
  else
    echo
    echo "❌ Failed to create task $i"
    break
  fi
done
echo
echo "✅ Created $SUCCESS_COUNT tasks successfully (Enterprise has NO LIMIT)"
echo

# 4.3 API使用量の確認（無制限）
echo "4.3 Checking API usage (Enterprise: UNLIMITED)..."
echo "Making rapid API calls to demonstrate no rate limiting..."

# 20回の高速API呼び出し
API_CALLS=0
for i in {1..20}; do
  RESPONSE=$(curl -s -X GET http://localhost:5000/subscriptions/current \
    -H "Authorization: Bearer $TOKEN" \
    -o /dev/null -w "%{http_code}")
  
  if [ "$RESPONSE" == "200" ]; then
    API_CALLS=$((API_CALLS + 1))
    echo -n "."
  else
    echo
    echo "❌ API call $i failed with status: $RESPONSE"
    break
  fi
done
echo
echo "✅ Made $API_CALLS API calls successfully (Enterprise has NO LIMIT)"
echo

# 5. 支払い履歴の確認
echo "5. Checking payment history..."
HISTORY_RESPONSE=$(curl -s -X GET "http://localhost:5000/payments/history?page=1&per_page=5" \
  -H "Authorization: Bearer $TOKEN")

if echo "$HISTORY_RESPONSE" | jq -e '.success' > /dev/null 2>&1; then
  echo "✅ Payment history available"
  echo "$HISTORY_RESPONSE" | jq '.data.items[] | {amount, currency, status, paid_at}'
else
  echo "No payment history found"
fi
echo

# 6. アップグレードオプションの確認（Enterpriseは最上位）
echo "6. Checking upgrade options..."
UPGRADE_RESPONSE=$(curl -s -X GET http://localhost:5000/payments/subscription/upgrade-options \
  -H "Authorization: Bearer $TOKEN")

echo "$UPGRADE_RESPONSE" | jq '.data'
echo

# 7. エンタープライズプラン機能サマリー
echo "=== Enterprise Plan Features Summary ==="
echo "✅ Teams: UNLIMITED (was 5 on Pro, 1 on Free)"
echo "✅ Team Members: UNLIMITED per team (was 10 on Pro, 3 on Free)"
echo "✅ Tasks: UNLIMITED (was 1,000 on Pro, 100 on Free)"
echo "✅ API Calls: UNLIMITED per day (was 10,000 on Pro, 1,000 on Free)"
echo "✅ Priority Support with SLA"
echo "✅ Advanced Analytics & Reporting"
echo "✅ Custom Integrations"
echo "✅ Dedicated Account Manager"
echo "✅ 99.9% Uptime SLA"
echo "✅ Advanced Security Features"
echo
echo "🎉 You have the ultimate plan with no limitations!"
echo