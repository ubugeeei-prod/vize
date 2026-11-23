NF && !/^#/ && (!limit || emitted < limit) {
    print
    emitted++
}
