# Work Command

Execute a work plan from the todo folder.

## Usage
/work <filename>

## Instructions

1. Read the work plan from `todo/<filename>.md`
2. Parse the plan to identify all tasks and their dependencies
3. Use the TodoWrite tool to create a task list from the plan
4. Execute each task in the specified order:
   - Mark the current task as "in_progress"
   - Perform all actions listed for the task
   - Verify success criteria are met
   - Mark the task as "completed"
   - Move to the next task
5. Handle any errors or blockers:
   - If a task cannot be completed, keep it as "in_progress"
   - Create a new task describing what needs to be resolved
   - Ask the user for guidance if needed
6. Provide regular status updates as tasks are completed
7. When all tasks are complete, provide a summary of what was accomplished

## Execution Guidelines
- Follow the exact actions specified in the plan
- Use all available tools as needed (Read, Write, Edit, Bash, etc.)
- Test and verify each step before marking it complete
- If the plan references specific files or commands, execute them exactly as written
- Maintain the context from the original plan throughout execution