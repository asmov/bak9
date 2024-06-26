bak9
================================================================================
*Keep your $HOME safe*  

Bak9 is rotational backup system for workstation users.  

bak9
--------------------------------------------------------------------------------
[![Latest Version: bak]][crates.io:bak]

[Latest Version: bak]: https://img.shields.io/crates/v/bak9.svg
[crates.io:bak]: https://crates.io/crates/bak9

### Usage

`bak9 [OPTIONS] <COMMAND>`

#### Commands
- `backup`   Performs backups as configured
- `config`   Manages configuration
- `log`      Reviews logs
- `summary`  Reviews a summary of recent backups

#### `bak9 backup` `<SUBCOMMAND> <NAME>`

`NAME`: The name of the backup configuration to operate on.

##### Subcommands:
- `scheduled` Performs backups as schedled
- `full` Manually performs a full backup
- `incremental` Manually performs an incremental backup

#### `bak9 config` `<SUBCOMMAND>`

##### Subcommands
- `setup` Initializes the user's bak9 configuration
- `edit` Opens the bak9 configuration file in their editor
- `verify` Verifies configuration
- `show` Displays the configuration file contents





License (GPL3)
--------------------------------------------------------------------------------
bak9: Rotational backup system for workstation users  
Copyright (C) 2024 Asmov LLC  

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a [copy](./LICENSE.txt) of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.

