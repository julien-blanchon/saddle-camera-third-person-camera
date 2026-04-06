# `saddle-camera-third-person-camera-lab`

Crate-local BRP and E2E verification harness for the generic camera core plus the optional action adapters.

## Purpose

- richer scene for obstruction, shoulder parity, lock-on, and retargeting checks
- reproducible E2E scenarios for the shared crate
- BRP-friendly runtime inspection without using project sandboxes
- interactive local controls for retargeting and cursor-lock debugging

## How To Run

```bash
cargo run -p saddle-camera-third-person-camera-lab
```

With E2E:

```bash
cargo run -p saddle-camera-third-person-camera-lab --features e2e -- third_person_camera_smoke
```

With BRP:

```bash
uv run --project .codex/skills/bevy-brp/script brp app launch saddle-camera-third-person-camera-lab
```

Helpful controls:

- `T` retargets the camera between the primary and alternate movers
- `F` toggles lock-on
- `E` cycles to the next lock-on candidate
- `Z` cycles to the previous lock-on candidate
- `C` swaps shoulders
- right mouse button enters aim while held
- `R` recenters
- `Q` toggles cursor lock

## Findings

- The lab keeps the target motion in `PostUpdate` before `ThirdPersonCameraSystems::UpdateIntent` so late authoritative movement does not jitter the camera.
- The scene includes narrow walls, pillars, and a low beam to make spring-arm pull-in and release visible.
