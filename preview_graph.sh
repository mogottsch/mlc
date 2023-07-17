#!/bin/bash

DOT_FILENAME="data/graph.dot"
SVG_FILENAME="data/graph.svg"

inotifywait -m -e modify $DOT_FILENAME | while read -r directory event filename; do
    echo "Regenerating svg"
    dot -Tsvg $DOT_FILENAME -o $SVG_FILENAME
    # killall feh
    # feh $SVG_FILENAME &
done
