<h2> <em><span style="color:#c52700">t</span>calc </em> </h2>

A CLI for time arithmetic. Inspired by [when](https://github.com/mitsuhiko/when/tree/main).
Try it [online](https://domenicocinque.github.io/tcalc/).

**Examples:**

* `2023/12/25 - 7d` → subtracts 7 days from December 25, 2023
* `2am + 30m` → adds 30 minutes to 2:00 AM
* `today - 2025/12/25` -> days until December 25, 2025
* `2024/04/27 + 40wd` → adds 40 working days, skipping weekends

## Usage

Run without installing: `cargo run -p cli -- "2am + 30m"`

### Syntax

* Dates use `YYYY/MM/DD` and can include time as `YYYY/MM/DD HH:MM`.
* Times accept 24-hour `HH:MM` or `H[am|pm]` forms (`2pm` → 14:00).
* Keywords: `today`, `tomorrow`, `yesterday`, `now`.
* Durations combine a number with a unit: `y`, `year`, `month`, `day|d`, `workingday|workday|wd`, `hour|h`, `minute|m`, `second|s`.
* Working days skip Saturdays and Sundays.
* Combine values with `+` and `-`, chaining operations left-to-right (`today - 2h + 30m`).
