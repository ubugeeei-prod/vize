match($0, /^(read|write|writev)\([01],/) {
    op = substr($0, RSTART, RLENGTH - 1) ")"
    size = $NF + 0
    calls[op]++
    bytes[op] += size
    if (size < 4096) small[op]++
}
END {
    for (op in calls) {
        printf "| %s | %s | %s | %d | %.1f | %d |\n", corpus, side, op, calls[op], bytes[op] / 1048576, small[op]
    }
}
