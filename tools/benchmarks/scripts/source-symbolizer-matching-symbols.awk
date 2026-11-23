$0 ~ (" [tT] .*" symbol) {
    print
    found = 1
}
END { if (!found) exit 1 }
