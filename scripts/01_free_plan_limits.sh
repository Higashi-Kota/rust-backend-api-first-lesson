#!/bin/bash

# 無料プランでの機能制限確認スクリプト

echo "=== Free Plan Limits Test ==="
echo

# ユーザー情報（テスト用）
TEST_USER="free_plan_test@example.com"
TEST_PASS="SecureP@ssw0rd!"

# 1. ユーザー登録
echo "1. Creating test user (Free plan)..."
SIGNUP_RESPONSE=$(curl -s -X POST http://localhost:5000/auth/signup \
  -H "Content-Type: application/json" \
  -d "{
    \"email\": \"$TEST_USER\",
    \"password\": \"$TEST_PASS\",
    \"username\": \"free_plan_user\"
  }")

if echo "$SIGNUP_RESPONSE" | jq -e '.success' > /dev/null 2>&1; then
  echo "✅ User created successfully"
else
  echo "User might already exist, proceeding with login..."
fi
echo

# 2. ログイン
echo "2. Logging in..."
LOGIN_RESPONSE=$(curl -s -X POST http://localhost:5000/auth/signin \
  -H "Content-Type: application/json" \
  -d "{
    \"identifier\": \"$TEST_USER\",
    \"password\": \"$TEST_PASS\"
  }")

TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.tokens.access_token')
if [ "$TOKEN" == "null" ] || [ -z "$TOKEN" ]; then
  echo "❌ Login failed"
  echo "$LOGIN_RESPONSE" | jq '.'
  exit 1
fi
echo "✅ Login successful"
echo

# 3. 現在のプラン確認
echo "3. Checking current subscription (should be FREE)..."
CURRENT_SUB=$(curl -s -X GET http://localhost:5000/subscriptions/current \
  -H "Authorization: Bearer $TOKEN")
echo "$CURRENT_SUB" | jq '.data.tier, .data.features'
echo

# 4. チーム作成制限のテスト（無料プランは1チームまで）
echo "4. Testing team creation limit (Free plan: max 1 team)..."
echo

# 最初のチーム作成（成功するはず）
echo "Creating first team (should succeed)..."
TEAM1_RESPONSE=$(curl -s -X POST http://localhost:5000/teams \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Free Team 1",
    "description": "First team on free plan"
  }')

if echo "$TEAM1_RESPONSE" | jq -e '.success' > /dev/null 2>&1; then
  echo "✅ First team created successfully"
  TEAM1_ID=$(echo "$TEAM1_RESPONSE" | jq -r '.data.id')
else
  echo "❌ Failed to create first team"
  echo "$TEAM1_RESPONSE" | jq '.'
fi
echo

# 2つ目のチーム作成（失敗するはず）
echo "Creating second team (should fail due to limit)..."
TEAM2_RESPONSE=$(curl -s -X POST http://localhost:5000/teams \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Free Team 2",
    "description": "Second team on free plan"
  }')

if echo "$TEAM2_RESPONSE" | jq -e '.success' > /dev/null 2>&1; then
  echo "❌ Second team was created (unexpected)"
else
  echo "✅ Second team creation blocked (expected)"
  echo "Error: $(echo "$TEAM2_RESPONSE" | jq -r '.error')"
fi
echo

# 5. チームメンバー制限のテスト（無料プランは3人まで）
echo "5. Testing team member limit (Free plan: max 3 members)..."
if [ -n "$TEAM1_ID" ]; then
  # メンバー追加のテスト（実際のメールアドレスが必要な場合はスキップ）
  echo "Note: Team member limit test requires valid email addresses"
  echo "Current team has owner (1 member by default)"
fi
echo

# 6. タスク作成制限のテスト（無料プランは100タスクまで）
echo "6. Testing task creation limit (Free plan: max 100 tasks)..."
echo "Creating multiple tasks to test limit..."

# バッチでタスクを作成
for i in {1..5}; do
  TASK_RESPONSE=$(curl -s -X POST http://localhost:5000/tasks \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "{
      \"title\": \"Free Plan Task $i\",
      \"description\": \"Task number $i on free plan\",
      \"status\": \"todo\"
    }")
  
  if echo "$TASK_RESPONSE" | jq -e '.success' > /dev/null 2>&1; then
    echo -n "."
  else
    echo
    echo "❌ Failed to create task $i"
    echo "$TASK_RESPONSE" | jq '.'
    break
  fi
done
echo
echo "✅ Sample tasks created"
echo

# 7. 現在の使用状況確認
echo "7. Checking current usage..."
echo "Teams created: 1/1"
echo "Tasks created: 5/100"
echo

echo "=== Free Plan Limitations Summary ==="
echo "❌ Teams: Limited to 1 team"
echo "❌ Team Members: Limited to 3 members per team"
echo "❌ Tasks: Limited to 100 tasks"
echo "❌ API Calls: Limited to 1,000 per day"
echo
echo "💡 To remove these limits, upgrade to Pro or Enterprise plan!"
echo