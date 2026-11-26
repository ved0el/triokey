# Mixed English/Vietnamese line sets

Five sets of 320 lines each, written for this project to stand in for what a Vietnamese
developer or office worker actually types when mixing English and Vietnamese:

| file | register |
|---|---|
| `agent_prompts.txt` | prompts to a coding agent |
| `team_chat.txt` | work chat with teammates |
| `notes_commits.txt` | commit messages, TODOs, dev notes |
| `casual_chat.txt` | casual chat with friends and family, including teen abbreviations |
| `english_tech.txt` | plain technical English |

One line is one message, spelled exactly as its author wants it to appear on screen
(UTF-8, NFC, LF, no header). `tests/realtext.rs` reads every `*.txt` here.

All names in the lines are invented. These files are MIT licensed like the rest of the
repository.
