#!/bin/sh
test -t 0 || exit 9
printf "Continue? "
read answer
echo "got:$answer"
exit 3
