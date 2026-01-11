---
name: skill-creator
description: Create reusable agent skills for OpenCode with proper structure, frontmatter, and validation
license: MIT
compatibility: opencode
metadata:
  audience: developers
  workflow: plugin-development
---
## What I do

I help you design, structure, and create OpenCode Agent Skills that agents can discover and load on-demand. I guide you through the entire skill creation workflow.

## When to use me

Use this skill when you need to:
- Create a new reusable agent instruction set
- Package best practices into a discoverable unit
- Share common workflows across your team
- Build Claude-compatible skills for interoperability

## Directory Structure

```
.opencode/
  skill/
    <skill-name>/
      SKILL.md
```

Or in global config:
```
~/.config/opencode/skill/<skill-name>/SKILL.md
```

Or Claude-compatible paths:
```
.claude/skills/<skill-name>/SKILL.md
~/.claude/skills/<skill-name>/SKILL.md
```

## Frontmatter Reference

```yaml
---
name: my-skill           # Required: 1-64 chars, lowercase alphanumeric with hyphens
description: What this skill does  # Required: 1-1024 chars
license: MIT             # Optional: SPDX license identifier
compatibility: opencode  # Optional: target platform
metadata:                # Optional: custom key-value pairs
  audience: developers
  workflow: documentation
---
```

## Name Validation Rules

Your skill name must match this regex:
```
^[a-z0-9]+(-[a-z0-9]+)*$
```

| Valid | Invalid |
|-------|---------|
| `doc-generator` | `Doc-Generator` (uppercase) |
| `api-client` | `api client` (space) |
| `react-component` | `react--component` (double hyphen) |
| `v1` | `-prefix` (starts with hyphen) |
| `test-123` | `suffix-` (ends with hyphen) |

## Permission Configuration

In `opencode.json`:
```json
{
  "permission": {
    "skill": {
      "my-skill": "allow",
      "experimental-*": "ask",
      "internal-*": "deny",
      "*": "allow"
    }
  }
}
```

Permission behaviors:
- `allow`: Agent loads skill immediately
- `deny`: Skill hidden, access rejected
- `ask`: User prompted for approval

## Complete Example

File: `.opencode/skill/code-review/SKILL.md`

```yaml
---
name: code-review
description: Perform thorough code reviews focusing on security, performance, and maintainability
license: MIT
compatibility: opencode
metadata:
  audience: developers
  workflow: quality-assurance
---
## What I do

I analyze code changes and provide constructive feedback on:
- Security vulnerabilities (injection, auth, secrets)
- Performance bottlenecks (N+1 queries, unnecessary loops)
- Code smells and maintainability issues
- Test coverage and edge cases

## When to use me

Use this when reviewing pull requests, merge requests, or before committing changes.

## My workflow

1. Understand the context and scope of changes
2. Identify critical paths and security-sensitive code
3. Check for common anti-patterns
4. Verify test coverage for new functionality
5. Provide actionable feedback with examples

## What I won't do

- Review generated code without context
- Approve changes without human review
- Ignore architectural constraints

## Interaction style

- Be specific: point to exact lines
- Be constructive: suggest improvements
- Be factual: cite best practices
```

## Skill Loading Process

1. Agent calls `skill({ name: "my-skill" })`
2. OpenCode searches configured skill paths
3. Frontmatter is parsed and validated
4. Full SKILL.md content is returned to agent
5. Agent incorporates skill instructions into context

## Best Practices

1. **Single responsibility**: Each skill should do one thing well
2. **Clear description**: Help agents decide when to use you
3. **Specific instructions**: Avoid ambiguity in behavior
4. **Examples**: Show rather than tell
5. **Interaction style**: Define how the skill communicates
6. **Scope boundaries**: Clarify what you won't do

## Interoperability

OpenCode skills are compatible with Claude's skill format. You can:
- Use same skill files for both platforms
- Share skills across OpenCode and Claude installations
- Migrate existing Claude skills without modification

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Skill not discovered | Check `SKILL.md` spelling (all caps) |
| Name validation fails | Verify regex pattern compliance |
| Frontmatter ignored | Ensure `---` delimiters are present |
| Permission denied | Check `opencode.json` permission rules |
| Duplicate name | Ensure unique names across all locations |

## Quick Reference

```yaml
---
name: my-new-skill
description: A clear, specific description
---
## What I do
Brief capability summary

## When to use me
Usage context

## My workflow
Step-by-step process

## Interaction style
How I communicate
```
