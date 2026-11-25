/^    [^[:space:]]/ {
    entry = substr($0, 5)
    separator = index(entry, ": ")
    if (!separator) next
    name = substr(entry, 1, separator - 1)
    gsub(/^"|"$/, "", name)
    if (name != package_name) next
    version = substr(entry, separator + 2)
    sub(/[[:space:]]+#.*/, "", version)
    sub(/[[:space:]]+$/, "", version)
    gsub(/^"|"$/, "", version)
    print version
    exit
}
