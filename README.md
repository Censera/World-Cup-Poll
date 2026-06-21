# World Cup Polls Bot (wcp)

A specialized Discord bot. The bot enables live match score predictions for football/soccer events and locks options automatically at kickoff.

## Features

- __Match Thread Generation__: Create polls for upcoming matches.
- __Prediction__: Users prepare individual selections via dropdown components before submitting a hard commit to state.
- __Automatic Kickoff Expiry__: The interaction engine auto-locks score updates the moment the global epoch counter bypasses the scheduled kickoff timestamp.
- __Dynamic Point Auditing__:
    - __3 Points__: Granted for users who predicted the exact score of the match.
    - __1 Point__: Granted for users who predicted the Win/Loss/Draw direction matching.

## Manual

This guide contains the exact operational workflow, command parameters, and instructions for managing and finalizing match predictions.

### Required Permissions

- __Manage Messages__: (`MANAGE_MESSAGES`)
- __Manage Events__: (`MANAGE_EVENTS`)

### Setup

#### To run a match prediction from start to finish, you must follow these three phases in order:
  
  1. __Before Kickoff__: Use the `/poll_match` command to spawn the interaction card in your target match channel.
  2. __At Kickoff__: You do __not__ need to manually delete or close the poll when the match begins. The bot handles this automatically.
  3. __Post-Match__: Once the final whistle blows and the game concludes, copy the Message ID of the original poll card and run the `/set_match_score` command to audit points.

#### Command reference:

`/poll_match`: Creates a live interactive prediction card for a fixture.
- Arguments
  - `team_a` `[String]`: The name (and optional flag emoji) of the home/first team.
  - `team_b` `[String]`: The name (and optional flag emoji) of the away/second team.
  - `kickoff` `[String]`: The exact start time matching a strict format schema: `YYYY/MM/DD HH:MM` (24-hour clock).

- Example Usage

```
/poll_match team_a:🇲🇦 Morocco team_b:🇫🇷 France kickoff:2026/06/25 18:00
```

`//set_match_score`: Closes out a prediction thread, logs points, and archives the interactive card elements.
- Arguments
  - `message_id_str` `[String]`: Discord Message ID of the poll card.
  - `score_a` `[Integer]`: Final goals scored by Team A.
  - `score_b` `[Integer]`: Final goals scored by Team B.

For the Message ID you need to enable `Developer Mode` in the settings.

- Example Usage

```
/set_match_score message_id_str:125839485729304958 score_a:2 score_b:1
```

## Installation & Configuration

### Prerequisites

Ensure you have the Rust toolchain installed (`rustc`, `cargo` 1.74+ recommended).

### Environment Layout

The system reads authentication tokens directly from the process environment variables:

```bash
export D_TOKEN="your_discord_bot_application_token_here"
```
