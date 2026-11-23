#!/bin/sh
if [ "$1" = "--api" ]; then exec sleep 600; fi
echo "error TS2688: Cannot find type definition file for 'vite/client'."
exit 2
