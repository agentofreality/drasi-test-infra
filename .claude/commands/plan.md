# Plan Command

Create a structured work plan and save it to the todo folder.

## Usage
/plan <filename>

## Instructions

1. Analyze the current context and user requirements
2. Create a comprehensive work plan with clear, actionable tasks
3. Save the plan to `todo/<filename>.md` with the following structure:

```markdown
# Work Plan: <Title>

## Overview
Brief description of the goal and scope

## Tasks

### 1. <Task Name>
- **Description**: What needs to be done
- **Actions**: 
  - Specific step 1
  - Specific step 2
  - ...
- **Success Criteria**: How to verify completion
- **Dependencies**: Prerequisites or related tasks

### 2. <Next Task>
...

## Execution Order
1. Task 1
2. Task 2 (depends on Task 1)
...

## Notes
- Any special considerations
- Potential challenges
- Required tools or resources
```

3. Use the TodoWrite tool to track the main tasks from the plan
4. Confirm the plan has been saved to `todo/<filename>.md`

The plan should be written in a way that's clear and executable by Claude Code when using the /work command.