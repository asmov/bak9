bak9
================================================================================
[![Latest Version]][crates.io]

[Latest Version]: https://img.shields.io/crates/v/bak9.svg
[crates.io]: https://crates.io/crates/bak9

Creates a backup `.bak` copy of a file.


Usage
--------------------------------------------------------------------------------

`bak [OPTIONS] FILE [DIR] [COMMAND]`

Creates a backup `.bak` copy of **FILE**.

If **DIR** is not specified, the copy is created in the same directory as FILE.

If DIR is specifed as `-`, or if the user lacks permissions to copy to DIR, the
user's app data directory will be used instead.

If *multiple* backups of FILE exist, the rotating filename extension used will be: `.bak.N`.

The most recent rotating backup will be always `.bak.0`. 

Pruning of rotating backups occurs after `-n NUM` backups. 

If the current backup is no different than its predecessor, copying will be skipped. 

Additional **COMMAND**s may be appended to list, compare, or delete backups.

### Options

- `-n NUM`  
Creates at most **NUM** backup files. [default: 10] 

- `-q`  
Quiet. Suppresses output.

- `-f`
Force the operation without confirmation.

### Commands

- `ls`  
Lists all backups of FILE in DIR.

- `diff N`  
Shows the differences of FILE and its `bak.N` copy in DIR. [default: 0]

- `rm`  
Deletes all backups of FILE in DIR.

License (GPL3)
--------------------------------------------------------------------------------
bak9: Crates a backup `.bak` copy of a file.  
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

