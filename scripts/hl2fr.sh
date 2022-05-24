#!/bin/bash
kindle_hl2fr_path=""
output_format="org" # "org" or "json"

if [ "$#" -ne 1 ]; then
  echo "Usage: $(basename "${BASH_SOURCE[0]}") <My Clippings path>"
elif [ -z "$kindle_hl2fr_path" ]; then
  echo "'kindle_hl2fr_path' must be set inside the script"
else
  $kindle_hl2fr_path "$1" 2>/dev/null | fzf -m | xargs -0 $kindle_hl2fr_path "$1" $output_format
fi

