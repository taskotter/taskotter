<p align="center">
  <img src="docs/assets/banner.png" alt="TaskOtter" width="100%">
</p>

<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/assets/logo-transparent-mode.png">
  <source media="(prefers-color-scheme: light)" srcset="docs/assets/logo-transparent-mode.png">
  <img alt="TaskOtter" src="docs/assets/logo-transparent-mode.png" width="50">
</picture>

# TaskOtter

**A collaborative workspace for people, agents, and the work between them.**

</div>

TaskOtter is an early-stage application for coordinating work with AI agents. It is designed around the idea that useful AI work rarely happens in a single prompt: it needs context, permissions, follow-up, review, automation, and a place where results can become real deliverables.

TaskOtter aims to provide that place.

## What TaskOtter Is For

TaskOtter helps teams organize AI-assisted work across conversations, tasks, agents, and workflows. The goal is to make agent collaboration feel less like a scattered set of chats and more like an operating system for getting work done.

The project is currently focused on:

- Human and AI agent collaboration
- Task and issue-oriented work tracking
- Reusable agent and skill configuration
- Controlled integrations with external tools
- Workflow automation for repeated operations
- Visibility into activity, usage, and outcomes

## Project Status

TaskOtter is under active private development. Public materials are intentionally limited while the product direction is being shaped.

More details will be shared as the project matures.

## Development

The repository currently contains the first Rust control-plane foundation in
`crates/taskotter-control-plane`.

Useful commands:

```sh
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p taskotter-control-plane
```

The service binds to `127.0.0.1:8080` by default. Set `TASKOTTER_BIND_ADDR` to
override the local bind address.

Initial contract endpoints:

- `GET /health`
- `GET /openapi.json`
- `POST /v1/working-groups`
- `POST /v1/issues`
- `POST /v1/comments`
- `POST /v1/registry`
- `POST /v1/policy/decisions`
- `POST /v1/usage/evaluate`
- `POST /v1/usage/events`
- `POST /v1/remote/usage-reports`
- `POST /v1/audit/events`

The generated OpenAPI document is the MVP source of truth for policy decisions,
gateway usage/audit events, and remote usage reports. See
`docs/contracts/mvp-contracts.md` for the cross-repository contract boundary and
publication path.

Architecture notes:

- `docs/module-boundaries.md` defines frontend and generated-contract dependency
  boundaries.
- `docs/integration-assisted-setup-ux-spec.md` defines the UX/spec boundary for
  proposing agents, skills, routes, and workspace defaults from external
  metadata before live credential or webhook implementation.
- `docs/i18n-architecture.md` defines the repository-local internationalization
  architecture, translation resource convention, locale precedence, API message
  policy, and localization QA gates.
- `docs/localization-release-readiness.md` defines the release checklist and PR
  readiness evidence for localization-impacting changes.

## License

This project is licensed under the PolyForm Strict License 1.0.0.

This project is not **open source**. The source code is made available for limited non-commercial personal use, research, experimentation, testing, and proof-of-concept evaluation only. Commercial use, redistribution, sublicensing, modification, creation of derivative works, and use for patent assertion are not permitted except where expressly allowed by the license.

See the LICENSE file for the full license text.
