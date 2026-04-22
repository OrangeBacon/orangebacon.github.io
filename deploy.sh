# this assumes that the /docs folder has been made a git worktree using
# `git worktree add docs gh-pages`

cd docs
git add .
git commit -m "Deploy"
git push
