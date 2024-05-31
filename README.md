# Genarr

Radically tiny generational array implementation. The
way it differs from (slotmap)[https://docs.rs/slotmap/latest/slotmap/]
is both in size and the fact that it uses untyped 
keys/indices - thus sort of eliminating the need for secondary maps.

I published this one because I've been copying this file
from project to project and thought it'd be nice to have it
in one place and maybe share it for those who'd find it useful.

## WARNING

It's not the most optimal way to implement a generational array, 
but it does it's job and is easy to use which are the goals of this crate.
