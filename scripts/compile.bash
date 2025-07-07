#! /usr/bin/env bash
find -type f -path "*/asset/*.class" -exec rm {} \;
find -type f -path "*/asset/*.java" | xargs -t -I {} javac {}