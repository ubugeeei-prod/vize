@REM DO NOT EDIT. Generated from tools/config/pkl/scripts/crates/vize/tests/programs/build-relative-musea.cmd.pkl.
@REM Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
@echo off
if not "%1"=="build" exit /b 9
if not "%VIZE_MUSEA_STATIC_BUILD%"=="1" exit /b 1
echo vite-plus stdout sentinel
mkdir dist\__musea__
echo ^<!doctype html^>^<title^>Musea^</title^> > dist\__musea__\index.html
