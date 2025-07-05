#!/bin/bash

# サブスクリプションキャンセル後の機能制限確認スクリプト

echo "=== Subscription Cancellation Test ==="
echo

# ユーザー情報（有料プランのユーザーを想定）
TEST_USER="cancel_test@example.com"
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
  echo "❌ Login failed"
  echo "This test requires a user with an active paid subscription"
  echo "Please run 02_pro_plan_upgrade.sh first to create a Pro user"
  exit 1
fi
echo "✅ Logged in successfully"
echo

# 2. 現在のサブスクリプション確認
echo "2. Checking current subscription..."
CURRENT_SUB=$(curl -s -X GET http://localhost:5000/subscriptions/current \
  -H "Authorization: Bearer $TOKEN")
CURRENT_TIER=$(echo "$CURRENT_SUB" | jq -r '.data.tier')
echo "Current plan: $CURRENT_TIER"
echo "$CURRENT_SUB" | jq '.data.features'
echo

# 3. 有料プランの場合、キャンセル方法を案内
if [ "$CURRENT_TIER" == "pro" ] || [ "$CURRENT_TIER" == "enterprise" ]; then
  echo "3. To cancel subscription:"
  echo
  
  # カスタマーポータルURL取得
  PORTAL_RESPONSE=$(curl -s -X POST http://localhost:5000/payments/portal \
    -H "Authorization: Bearer $TOKEN")
  
  if echo "$PORTAL_RESPONSE" | jq -e '.success' > /dev/null 2>&1; then
    PORTAL_URL=$(echo "$PORTAL_RESPONSE" | jq -r '.data.portal_url')
    echo "📋 Steps to cancel:"
    echo "   1. Open customer portal: $PORTAL_URL"
    echo "   2. Click 'Cancel plan' or 'Cancel subscription'"
    echo "   3. Confirm cancellation"
    echo "   4. Wait for webhook to process"
    echo "   5. Re-run this script to test Free plan limits"
    echo
    echo "⚠️  After cancellation:"
    echo "   - You'll be immediately downgraded to Free plan"
    echo "   - Existing data will be retained"
    echo "   - Features will be limited based on Free plan"
  else
    echo "❌ Unable to get customer portal URL"
  fi
  echo
  exit 0
fi

# 4. 無料プランに戻った後の制限確認
echo "3. Testing Free plan restrictions after cancellation..."
echo

# 4.1 既存のリソース確認
echo "4.1 Checking existing resources..."

# チーム数確認
TEAMS_RESPONSE=$(curl -s -X GET http://localhost:5000/teams \
  -H "Authorization: Bearer $TOKEN")
TEAM_COUNT=$(echo "$TEAMS_RESPONSE" | jq -r '.data | length')
echo "Current teams: $TEAM_COUNT"

if [ $TEAM_COUNT -gt 1 ]; then
  echo "⚠️  Warning: You have $TEAM_COUNT teams but Free plan allows only 1"
  echo "   - You can still access all teams"
  echo "   - But cannot create new teams"
  echo "   - Consider deleting extra teams or upgrading again"
fi
echo

# タスク数確認
TASKS_RESPONSE=$(curl -s -X GET http://localhost:5000/tasks \
  -H "Authorization: Bearer $TOKEN")
TASK_COUNT=$(echo "$TASKS_RESPONSE" | jq -r '.data | length')
echo "Current tasks: $TASK_COUNT"

if [ $TASK_COUNT -gt 100 ]; then
  echo "⚠️  Warning: You have $TASK_COUNT tasks but Free plan allows only 100"
  echo "   - Existing tasks are retained"
  echo "   - But cannot create new tasks until under limit"
fi
echo

# 4.2 新規リソース作成の制限テスト
echo "4.2 Testing creation limits on Free plan..."

# 新しいチーム作成試行（すでに1つ以上ある場合は失敗するはず）
if [ $TEAM_COUNT -ge 1 ]; then
  echo "Attempting to create new team (should fail if at limit)..."
  NEW_TEAM_RESPONSE=$(curl -s -X POST http://localhost:5000/teams \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
      "name": "Post-cancellation Team",
      "description": "Testing Free plan limits after downgrade"
    }')
  
  if echo "$NEW_TEAM_RESPONSE" | jq -e '.success' > /dev/null 2>&1; then
    echo "✅ Team created (was under limit)"
  else
    echo "❌ Team creation blocked (at Free plan limit)"
    echo "Error: $(echo "$NEW_TEAM_RESPONSE" | jq -r '.error')"
  fi
fi
echo

# 新しいタスク作成試行
echo "Attempting to create new task..."
NEW_TASK_RESPONSE=$(curl -s -X POST http://localhost:5000/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Post-cancellation Task",
    "description": "Testing task creation after downgrade",
    "status": "todo"
  }')

if echo "$NEW_TASK_RESPONSE" | jq -e '.success' > /dev/null 2>&1; then
  echo "✅ Task created (under 100 task limit)"
else
  echo "❌ Task creation blocked (at Free plan limit)"
  echo "Error: $(echo "$NEW_TASK_RESPONSE" | jq -r '.error')"
fi
echo

# 5. アップグレードオプションの確認
echo "5. Available upgrade options after cancellation..."
TIERS_RESPONSE=$(curl -s -X GET http://localhost:5000/payments/subscription/tiers \
  -H "Authorization: Bearer $TOKEN")

echo "$TIERS_RESPONSE" | jq '.data[] | select(.id != "free") | {tier: .id, price: .price, features: .features}'
echo

# 6. サマリー
echo "=== Post-Cancellation Summary ==="
echo "📊 Current Status:"
echo "   - Plan: FREE (downgraded from $CURRENT_TIER)"
echo "   - Teams: $TEAM_COUNT/1 allowed"
echo "   - Tasks: $TASK_COUNT/100 allowed"
echo
echo "⚠️  Restrictions Applied:"
echo "   - Cannot create more than 1 team"
echo "   - Cannot create more than 100 tasks"
echo "   - Limited to 3 members per team"
echo "   - Limited to 1,000 API calls per day"
echo
echo "💡 Options:"
echo "   1. Delete excess resources to fit Free plan limits"
echo "   2. Upgrade back to Pro or Enterprise"
echo "   3. Export your data before it exceeds retention period"
echo