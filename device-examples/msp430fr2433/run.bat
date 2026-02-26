@REM echo off
set arg1=%1
@REM mspdebug --allow-fw-update tilib "prog %arg1%"
@REM mspdebug --allow-fw-update tilib "run"

@REM mspdebug --allow-fw-update tilib
@REM prog %arg1%
@REM run

echo prog %arg1% > temp_commands.txt
echo run >> temp_commands.txt

mspdebug --allow-fw-update tilib < temp_commands.txt
del temp_commands.txt
