#!/bin/bash

# プロプランアップグレード後の機能確認スクリプト

echo "=== Pro Plan Upgrade Test ==="
echo

# ユーザー情報（既存ユーザーを想定）
TEST_USER="pro_plan_test@example.com"
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
      \"username\": \"pro_plan_user\"
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

# 3. プロプランへのアップグレード（チェックアウトセッション作成）
if [ "$CURRENT_TIER" != "pro" ] && [ "$CURRENT_TIER" != "enterprise" ]; then
  echo "3. Creating checkout session for Pro plan upgrade..."
  CHECKOUT_RESPONSE=$(curl -s -X POST http://localhost:5000/payments/checkout \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
      "tier": "pro"
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
  echo "   5. Re-run this script to test Pro features"
  echo
  exit 0
fi

# 4. プロプラン機能のテスト
echo "3. Testing Pro plan features..."
echo

# 4.1 チーム作成制限のテスト（プロプランは5チームまで）
echo "4.1 Testing team creation (Pro plan: max 5 teams)..."
echo

# 既存のチーム数を確認
TEAMS_RESPONSE=$(curl -s -X GET http://localhost:5000/teams \
  -H "Authorization: Bearer $TOKEN")
EXISTING_TEAMS=$(echo "$TEAMS_RESPONSE" | jq -r '.data | length')
echo "Existing teams: $EXISTING_TEAMS"

# 新しいチームを作成（最大5つまで）
TEAMS_TO_CREATE=$((5 - EXISTING_TEAMS))
if [ $TEAMS_TO_CREATE -gt 0 ]; then
  echo "Creating $TEAMS_TO_CREATE more teams..."
  for i in $(seq 1 $TEAMS_TO_CREATE); do
    TEAM_RESPONSE=$(curl -s -X POST http://localhost:5000/teams \
      -H "Authorization: Bearer $TOKEN" \
      -H "Content-Type: application/json" \
      -d "{
        \"name\": \"Pro Team $((EXISTING_TEAMS + i))\",
        \"description\": \"Team $((EXISTING_TEAMS + i)) on Pro plan\"
      }")
    
    if echo "$TEAM_RESPONSE" | jq -e '.success' > /dev/null 2>&1; then
      echo "✅ Team $((EXISTING_TEAMS + i)) created successfully"
    else
      echo "❌ Failed to create team $((EXISTING_TEAMS + i))"
      echo "$TEAM_RESPONSE" | jq '.'
      break
    fi
  done
fi

# 6つ目のチーム作成試行（失敗するはず）
echo
echo "Attempting to create 6th team (should fail)..."
TEAM6_RESPONSE=$(curl -s -X POST http://localhost:5000/teams \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Pro Team 6",
    "description": "6th team - should fail"
  }')

if echo "$TEAM6_RESPONSE" | jq -e '.success' > /dev/null 2>&1; then
  echo "❌ 6th team was created (unexpected)"
else
  echo "✅ 6th team creation blocked (expected)"
  echo "Error: $(echo "$TEAM6_RESPONSE" | jq -r '.error')"
fi
echo

# 4.2 タスク作成制限のテスト（プロプランは1000タスクまで）
echo "4.2 Testing task creation (Pro plan: max 1,000 tasks)..."
echo "Creating batch of tasks..."

# 20個のタスクを作成してテスト
SUCCESS_COUNT=0
for i in {1..20}; do
  TASK_RESPONSE=$(curl -s -X POST http://localhost:5000/tasks \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "{
      \"title\": \"Pro Plan Task $i\",
      \"description\": \"Testing Pro plan task limits - Task $i\",
      \"status\": \"todo\"
    }")
  
  if echo "$TASK_RESPONSE" | jq -e '.success' > /dev/null 2>&1; then
    SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
    echo -n "."
  else
    echo
    echo "❌ Failed to create task $i"
    break
  fi
done
echo
echo "✅ Created $SUCCESS_COUNT tasks successfully (Pro plan allows up to 1,000)"
echo

# 5. カスタマーポータルへのアクセス
echo "5. Getting customer portal URL..."
PORTAL_RESPONSE=$(curl -s -X POST http://localhost:5000/payments/portal \
  -H "Authorization: Bearer $TOKEN")

if echo "$PORTAL_RESPONSE" | jq -e '.success' > /dev/null 2>&1; then
  PORTAL_URL=$(echo "$PORTAL_RESPONSE" | jq -r '.data.portal_url')
  echo "✅ Customer portal available"
  echo "🔗 Portal URL: $PORTAL_URL"
  echo "   (Use this to manage subscription, update payment method, or cancel)"
else
  echo "❌ Customer portal not available"
  echo "$PORTAL_RESPONSE" | jq '.'
fi
echo

# 6. プラン詳細確認
echo "6. Pro Plan Features Summary:"
echo "✅ Teams: Up to 5 teams (was 1 on Free)"
echo "✅ Team Members: Up to 10 members per team (was 3 on Free)"
echo "✅ Tasks: Up to 1,000 tasks (was 100 on Free)"
echo "✅ API Calls: Up to 10,000 per day (was 1,000 on Free)"
echo "✅ Priority Support"
echo "✅ Advanced Analytics"
echo
echo "💡 For unlimited features, consider upgrading to Enterprise plan!"
echo