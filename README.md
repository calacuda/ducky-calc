# DuckyCalc

Hardware [ducky-script](https://docs.hak5.org/hak5-usb-rubber-ducky/duckyscript-quick-reference/) interpreter that runs on a [ClockworkPi PicoCalc](https://www.clockworkpi.com/picocalc).

This project allows selecting from multiple ducky script payloads to inject keystrokes into a target computer.

## Demo Video

## Folder Structure

| **Directory** | **Whats There** |
|----|----|
| `ducky-exec/` | the ducky-script interpreter as a no-std rust crate. |
| `payloads/` | example payloads containing both metadata files (meta-data.txt) & ducky-script files (payload.txt) |
| `helpers/` | misc helper scripts. |

## Project Status

I would classify this project as in the "Minimum Viable Product" stage.
