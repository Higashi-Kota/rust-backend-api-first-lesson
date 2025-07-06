#!/bin/bash

# プラン別機能比較スクリプト

echo "=== Subscription Plan Comparison Test ==="
echo

# 3つのテストユーザーを定義
FREE_USER="free_comparison@example.com"
PRO_USER="pro_comparison@example.com"
ENTERPRISE_USER="enterprise_comparison@example.com"
TEST_PASS="SecureP@ssw0rd!"

# カラー定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# プラン別の制限値
declare -A LIMITS
LIMITS[free_teams]=1
LIMITS[pro_teams]=5
LIMITS[enterprise_teams]="UNLIMITED"
LIMITS[free_members]=3
LIMITS[pro_members]=10
LIMITS[enterprise_members]="UNLIMITED"
LIMITS[free_tasks]=100
LIMITS[pro_tasks]=1000
LIMITS[enterprise_tasks]="UNLIMITED"
LIMITS[free_api]=1000
LIMITS[pro_api]=10000
LIMITS[enterprise_api]="UNLIMITED"

# ヘルパー関数：ユーザー作成とログイン
login_or_create_user() {
  local email=$1
  local username=$2
  
  # ログイン試行
  LOGIN_RESPONSE=$(curl -s -X POST http://localhost:5000/auth/signin \
    -H "Content-Type: application/json" \
    -d "{
      \"identifier\": \"$email\",
      \"password\": \"$TEST_PASS\"
    }")
  
  TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.tokens.access_token')
  
  if [ "$TOKEN" == "null" ] || [ -z "$TOKEN" ]; then
    # ユーザー作成
    curl -s -X POST http://localhost:5000/auth/signup \
      -H "Content-Type: application/json" \
      -d "{
        \"email\": \"$email\",
        \"password\": \"$TEST_PASS\",
        \"username\": \"$username\"
      }" > /dev/null
    
    # 再度ログイン
    LOGIN_RESPONSE=$(curl -s -X POST http://localhost:5000/auth/signin \
      -H "Content-Type: application/json" \
      -d "{
        \"identifier\": \"$email\",
        \"password\": \"$TEST_PASS\"
      }")
    
    TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.tokens.access_token')
  fi
  
  echo "$TOKEN"
}

# ヘルパー関数：機能テスト
test_feature_limit() {
  local token=$1
  local feature=$2
  local endpoint=$3
  local payload=$4
  
  RESPONSE=$(curl -s -X POST "http://localhost:5000/$endpoint" \
    -H "Authorization: Bearer $token" \
    -H "Content-Type: application/json" \
    -d "$payload")
  
  if echo "$RESPONSE" | jq -e '.success' > /dev/null 2>&1; then
    echo "allowed"
  else
    echo "blocked"
  fi
}

# 1. ユーザーのセットアップ
echo "1. Setting up test users..."
echo

echo -n "Free user: "
FREE_TOKEN=$(login_or_create_user "$FREE_USER" "free_comparison_user")
if [ -n "$FREE_TOKEN" ] && [ "$FREE_TOKEN" != "null" ]; then
  echo -e "${GREEN}✓${NC}"
else
  echo -e "${RED}✗${NC}"
fi

echo -n "Pro user: "
PRO_TOKEN=$(login_or_create_user "$PRO_USER" "pro_comparison_user")
if [ -n "$PRO_TOKEN" ] && [ "$PRO_TOKEN" != "null" ]; then
  echo -e "${GREEN}✓${NC}"
  
  # Proプランの購入を促す
  CURRENT_SUB=$(curl -s -X GET http://localhost:5000/subscriptions/current \
    -H "Authorization: Bearer $PRO_TOKEN" | jq -r '.data.tier')
  
  if [ "$CURRENT_SUB" != "pro" ] && [ "$CURRENT_SUB" != "enterprise" ]; then
    echo -e "${YELLOW}  Note: User needs to upgrade to Pro plan${NC}"
  fi
else
  echo -e "${RED}✗${NC}"
fi

echo -n "Enterprise user: "
ENTERPRISE_TOKEN=$(login_or_create_user "$ENTERPRISE_USER" "enterprise_comparison_user")
if [ -n "$ENTERPRISE_TOKEN" ] && [ "$ENTERPRISE_TOKEN" != "null" ]; then
  echo -e "${GREEN}✓${NC}"
  
  # Enterpriseプランの購入を促す
  CURRENT_SUB=$(curl -s -X GET http://localhost:5000/subscriptions/current \
    -H "Authorization: Bearer $ENTERPRISE_TOKEN" | jq -r '.data.tier')
  
  if [ "$CURRENT_SUB" != "enterprise" ]; then
    echo -e "${YELLOW}  Note: User needs to upgrade to Enterprise plan${NC}"
  fi
else
  echo -e "${RED}✗${NC}"
fi
echo

# 2. 機能比較テーブルのヘッダー
echo "2. Feature Comparison Matrix"
echo
printf "%-30s %-15s %-15s %-15s\n" "Feature" "Free" "Pro" "Enterprise"
printf "%-30s %-15s %-15s %-15s\n" "==============================" "===============" "===============" "==============="

# 2.1 プラン情報
printf "%-30s ${GREEN}%-15s${NC} ${BLUE}%-15s${NC} ${YELLOW}%-15s${NC}\n" "Monthly Price" "\$0" "\$29" "\$99"

# 2.2 チーム制限
printf "%-30s %-15s %-15s %-15s\n" "Max Teams" "${LIMITS[free_teams]}" "${LIMITS[pro_teams]}" "${LIMITS[enterprise_teams]}"

# 2.3 メンバー制限
printf "%-30s %-15s %-15s %-15s\n" "Max Members per Team" "${LIMITS[free_members]}" "${LIMITS[pro_members]}" "${LIMITS[enterprise_members]}"

# 2.4 タスク制限
printf "%-30s %-15s %-15s %-15s\n" "Max Tasks" "${LIMITS[free_tasks]}" "${LIMITS[pro_tasks]}" "${LIMITS[enterprise_tasks]}"

# 2.5 API制限
printf "%-30s %-15s %-15s %-15s\n" "API Calls per Day" "${LIMITS[free_api]}" "${LIMITS[pro_api]}" "${LIMITS[enterprise_api]}"

# 2.6 機能比較
printf "%-30s %-15s %-15s %-15s\n" "File Upload" "✓ Basic" "✓ Advanced" "✓ Premium"
printf "%-30s %-15s %-15s %-15s\n" "Analytics" "✗" "✓ Basic" "✓ Advanced"
printf "%-30s %-15s %-15s %-15s\n" "Priority Support" "✗" "✓" "✓ + SLA"
printf "%-30s %-15s %-15s %-15s\n" "Custom Integrations" "✗" "✗" "✓"
printf "%-30s %-15s %-15s %-15s\n" "Dedicated Account Manager" "✗" "✗" "✓"
printf "%-30s %-15s %-15s %-15s\n" "Data Export" "CSV only" "CSV, JSON" "All formats"
printf "%-30s %-15s %-15s %-15s\n" "Audit Logs" "7 days" "30 days" "Unlimited"
printf "%-30s %-15s %-15s %-15s\n" "Uptime SLA" "✗" "99.5%" "99.9%"
echo

# 3. 実際の機能テスト
echo "3. Live Feature Testing"
echo

# 各プランのティア情報を取得
echo "Checking available tiers for each plan..."
for plan in "Free:$FREE_TOKEN" "Pro:$PRO_TOKEN" "Enterprise:$ENTERPRISE_TOKEN"; do
  IFS=':' read -r plan_name token <<< "$plan"
  if [ -n "$token" ] && [ "$token" != "null" ]; then
    echo -n "$plan_name plan tiers: "
    TIERS=$(curl -s -X GET http://localhost:5000/payments/subscription/tiers \
      -H "Authorization: Bearer $token" | jq -r '.data[].id' | tr '\n' ', ' | sed 's/,$//')
    echo "$TIERS"
  fi
done
echo

# 4. アップグレード促進
echo "4. Upgrade Recommendations"
echo

# Free → Pro アップグレード理由
echo -e "${BLUE}Why upgrade from Free to Pro?${NC}"
echo "  • 5x more teams (1 → 5)"
echo "  • 3x more team members (3 → 10)"
echo "  • 10x more tasks (100 → 1,000)"
echo "  • 10x more API calls (1,000 → 10,000)"
echo "  • Analytics and reporting features"
echo "  • Priority customer support"
echo

# Pro → Enterprise アップグレード理由
echo -e "${YELLOW}Why upgrade from Pro to Enterprise?${NC}"
echo "  • Unlimited everything - no restrictions"
echo "  • Custom integrations for your tools"
echo "  • Dedicated account manager"
echo "  • 99.9% uptime SLA guarantee"
echo "  • Advanced security features"
echo "  • Unlimited audit log retention"
echo

# 5. クイックアクション
echo "5. Quick Actions"
echo

# チェックアウトURLを生成
if [ -n "$FREE_TOKEN" ] && [ "$FREE_TOKEN" != "null" ]; then
  echo "🚀 Upgrade Free user to Pro:"
  CHECKOUT=$(curl -s -X POST http://localhost:5000/payments/checkout \
    -H "Authorization: Bearer $FREE_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"tier": "pro"}' 2>/dev/null)
  
  if echo "$CHECKOUT" | jq -e '.data.checkout_url' > /dev/null 2>&1; then
    CHECKOUT_URL=$(echo "$CHECKOUT" | jq -r '.data.checkout_url')
    echo "   $CHECKOUT_URL"
  else
    echo "   Run: ./02_pro_plan_upgrade.sh"
  fi
fi

echo
echo "📊 View detailed analytics (Pro/Enterprise only):"
echo "   curl -X GET http://localhost:5000/analytics/dashboard -H 'Authorization: Bearer <TOKEN>'"
echo
echo "💳 Manage subscription:"
echo "   curl -X POST http://localhost:5000/payments/portal -H 'Authorization: Bearer <TOKEN>'"
echo

# 6. テスト完了
echo
echo "=== Comparison Test Complete ==="
echo "ℹ️  Note: Some users may need to complete Stripe checkout to access paid features"
echo