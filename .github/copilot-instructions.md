# Copilot Instructions - Rust Micro Front-End Application

## 🚨 CRITICAL AI ASSISTANT BEHAVIORAL REQUIREMENTS

### Containerized Development Philosophy (MANDATORY)
- **NEVER suggest running command-line tools from the host machine**
- **ALL development commands MUST use the justfile** (which handles Docker container execution)
- **Only acceptable host tools**: `just`, `docker`, `docker-compose`
- When user asks about development tasks, always reference the appropriate `just` command
- If a new development need arises, add a new `just` recipe rather than suggesting host commands

### AI Assistant Workflow Requirements
1. **Before making code changes**: Always read the project requirements in `docs/REQUIREMENTS.md`
2. **Environment variables**: Reference `.env.example` for configuration details
3. **Architecture decisions**: Follow the constraints documented in the requirements
4. **Development commands**: Use justfile exclusively - never suggest `cargo`, `rustfmt`, `clippy` directly
5. **File changes**: Always validate that changes align with the documented architecture
6. **Gitignore maintenance**: Update `.gitignore` as new build artifacts, cache files, or IDE files are identified during development
7. **Incremental folder structure approach**: Don't plan folder structure ahead of time. At every increment, review and include only what's necessary at that step, and flesh out the rest as we go. Always discuss folder organization decisions with the user before creating new directories.
8. **No sprint terminology**: This project doesn't use sprint-based development. Use terms like "current phase", "next steps", or "current focus" instead.
9. **Work plan maintenance**: Update `docs/WORK_PLAN.md` as tasks are completed, moving items from pending to in-progress to completed. Update progress percentages and change log entries to reflect current development status.

### Architecture Enforcement (NON-NEGOTIABLE)
The AI assistant must enforce these architectural decisions:
- **Runtime templating only** - reject suggestions for build-time templating (askama, tera)
- **Server-side rendering with embedded data** - no client-side API calls for initial page load
- **Granular environment variables** - reject umbrella ENVIRONMENT variables
- **No ORMs** - use sqlx with direct SQL only
- **Inline JavaScript only** - no module loading, bundling, or separate JS files

### Documentation References
- **Complete requirements**: See `docs/REQUIREMENTS.md`
- **Environment configuration**: See `.env.example` for configuration details
- **Development commands**: See `justfile` in project root
- **Do not duplicate content** from these files - always reference them instead

### Code Quality Standards
- **Security first**: JWT validation only (no token generation), input sanitization, parameterized queries
- **Performance target**: Lighthouse 100/100 score
- **Testing philosophy**: Comprehensive unit, integration, and e2e tests using containerized test runner
- **Error handling**: Graceful degradation with structured logging

## AI Assistant Behavioral Guidelines

### When Implementing Features
1. Read existing code structure first
2. Validate against documented architecture constraints
3. Use containerized development commands exclusively
4. Implement with security and performance in mind
5. Add appropriate tests using the justfile test commands

### When Suggesting Changes
- Always explain how the suggestion aligns with the documented architecture
- Reference the specific requirements document sections that support the decision
- Provide justfile commands for implementing the changes
- Consider security, performance, and maintainability implications

### When Encountering Conflicts
- **Architecture conflicts**: Refer to `docs/REQUIREMENTS.md` for resolution
- **Environment conflicts**: Refer to `.env.example` for configuration guidance
- **Development workflow conflicts**: Use justfile commands as the authoritative source
