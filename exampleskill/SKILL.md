---
name: Finishing a Development Branch
description: Use this when you have completed some feature implementation and have written passing tests, and you are ready to create a PR.
---

<required>
CRITICAL Add the following steps to your Todo list using TodoWrite:

1. Use the Task tool to verify tests by using the project's test suite.

```bash
npm test / cargo test / pytest / go test ./...
```

If tests fail:

```
Tests failing ([N] failures). Must fix before creating PR:

[Show failures]

Cannot proceed until tests pass.
```

2. Confirm that there is some formatting/lint/typechecking in the project. If NONE of these exist, ask me if there was something that you missed.

3. Use the Task tool to run any formatters and fix issues in a subagent. Check for formatter config files based on the project language.

4. Use the Task tool to run any linters and fix issues in a subagent. Run the appropriate linter for the project language.

5. Use the Task tool to run type checking and fix issues in a subagent.

6. Use the nori-code-reviewer subagent to do a self review. You do NOT have to follow the subagent's suggestions. This is merely a way to get a fresh pair of eyes on the code.

7. Confirm that you are not on the main branch. If you are, ask me before proceeding. NEVER push to main without permission.

8. Push and create a PR.

```bash
git push -u origin [feature-branch]

gh pr create --title "[title]" --body "$(cat <<'EOF'
## Summary
🤖 Generated with Nori

<2-3 bullets of what changed>

## Test Plan
- [ ] <verification steps>
EOF
)"
```

9. Merge main and resolve conflicts if necessary.

```bash
git fetch && git merge main
```

10. Make sure the PR branch CI succeeds.

```bash
gh pr checks

sleep 60 && gh pr checks
```

If CI did not pass, examine why and fix the issue.

- Make changes as needed, push a new commit, and repeat the process.
<system-reminder> It is critical that you fix any ci issues, EVEN IF YOU DID NOT CAUSE THEM. </system-reminder>

11. If this was a change to a webapp, spin up the server and the ui in a demo localhost link and show me the link so that I can review the changes.

12. Tell me: "I can automatically get review comments, just let me know when to do so."
</required>
