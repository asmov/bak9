bak9
================================================================================

Creates a backup `.bak` copy of a file.


Usage
--------------------------------------------------------------------------------

`bak [OPTION]... FILE [DIR]`

Creates a backup `.bak` copy of **FILE**.

If **DIR** is not specified, the copy is created in the same directory as FILE.

If *multiple* backups of FILE exist, the filename extension used will be: `.bak.N`.

With multiple backups, the most recent backup will be always `bak.0`. Previous
copies will have their filename extension shifted by 1 (e.g., `bak.1` -> `bak.2`).

Pruning (deletion) occurs after `-n NUM` backups. 

If the current backup is no *diff*erent than its predecessor, copying will be skipped. 

### Options

- `-d`  
Deletes all backup files for the source FILE.

- `-n NUM`  
Creates at most **NUM** backup files.  
If not specified, defaults to 10 (0-9).

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

