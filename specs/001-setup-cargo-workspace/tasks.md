---
description: "Task list for Cargo Workspace and Basic Runner Setup"
---

# Tasks: Cargo Workspaceと基本ランナーのセットアップ

**Input**: Design documents from `/specs/001-setup-cargo-workspace/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/workspace.md, quickstart.md

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [ ] T001 Create root Cargo.toml defining the workspace and members in /Cargo.toml
- [ ] T002 [P] Create directory structure for `core` crate in core/
- [ ] T003 [P] Create directory structure for `runner-debian` crate in runner-debian/
- [ ] T004 [P] Create directory structure for `runner-wslg` crate in runner-wslg/

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

- [ ] T005 Create library definition for core crate in core/Cargo.toml
- [ ] T006 [P] Initialize basic library entry point in core/src/lib.rs
- [ ] T007 [P] Create Cargo.toml for runner-debian with dependency on core in runner-debian/Cargo.toml
- [ ] T008 [P] Create Cargo.toml for runner-wslg with dependency on core in runner-wslg/Cargo.toml

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Workspace構造の確立 (Priority: P1) 🎯 MVP

**Goal**: ADR 0002に準拠したCargo Workspace構造を確立し、ビルド可能であることを確認する。

**Independent Test**: `cargo build` がエラーなく完了すること。

### Implementation for User Story 1

- [ ] T009 [US1] Verify workspace members are correctly linked in /Cargo.toml
- [ ] T010 [US1] Validate directory layout against ADR 0002 in core/, runner-debian/, runner-wslg/

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 - Debianランナーの動作確認 (Priority: P2)

**Goal**: `runner-debian`クレートを実行し、「Hello, World!」が表示されることを確認する。

**Independent Test**: `cargo run -p runner-debian` を実行し、標準出力に期待されるメッセージが表示されること。

### Implementation for User Story 2

- [ ] T011 [US2] Implement "Hello, World!" main function in runner-debian/src/main.rs

**Checkpoint**: At this point, User Story 2 should be fully functional and testable independently

---

## Phase 5: User Story 3 - WSLgランナーの動作確認 (Priority: P3)

**Goal**: `runner-wslg`クレートを実行し、「Hello, World!」が表示されることを確認する。

**Independent Test**: `cargo run -p runner-wslg` を実行し、標準出力に期待されるメッセージが表示されること。

### Implementation for User Story 3

- [ ] T012 [US3] Implement "Hello, World!" main function in runner-wslg/src/main.rs

**Checkpoint**: At this point, User Story 3 should be fully functional and testable independently

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T013 Run quickstart.md validation to confirm all scenarios pass
- [ ] T014 Update project documentation if any structural deviations occurred

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 $\to$ P2 $\to$ P3)
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) - No dependencies on other stories

### Within Each User Story

- Models before services
- Services before endpoints
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel (T002, T003, T004)
- All Foundational tasks marked [P] can run in parallel (T006, T007, T008)
- Once Foundational phase completes, all user stories can start in parallel (T009-T012)

---

## Parallel Example: User Story 1

```bash
# User Story 1 doesn't have parallel implementation tasks within itself in this simple setup.
```

## Parallel Example: User Story 2 & 3

```bash
# User Story 2 and 3 can be implemented in parallel as they are independent binary crates:
Task: "Implement 'Hello, World!' main function in runner-debian/src/main.rs"
Task: "Implement 'Hello, World!' main function in runner-wslg/src/main.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test User Story 1 independently with `cargo build`

### Incremental Delivery

1. Complete Setup + Foundational $\to$ Foundation ready
2. Add User Story 1 $\to$ Test independently $\to$ Deploy/Demo (MVP!)
3. Add User Story 2 $\to$ Test independently $\to$ Deploy/Demo
4. Add User Story 3 $\to$ Test independently $\to$ Deploy/Demo

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1
   - Developer B: User Story 2
   - Developer C: User Story 3

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to la specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
