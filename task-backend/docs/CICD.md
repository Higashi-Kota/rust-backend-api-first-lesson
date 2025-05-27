# このプロジェクトでのDockerfile の役割

1. CI/CDでのイメージビルド (build-docker job)
2. docker-compose.yml での起動
3. 本番デプロイ用のコンテナ化

つまり「Rustアプリケーションを確実にコンテナで起動できる」ことだけが重要です。