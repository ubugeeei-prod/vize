#!/bin/sh
[ "$#" = 1 ] && [ "$1" = --version ] || exit 9
printf 'Version @@VERSION@@\n'
