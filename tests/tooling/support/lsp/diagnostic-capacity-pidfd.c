#define _GNU_SOURCE
#include <errno.h>
#include <limits.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/syscall.h>
#include <unistd.h>

// Keep the descriptor bound while checking the process's birth and executable.
static int inspect(int pid, const char *birth, const char *executable, int signum) {
    int fd = syscall(SYS_pidfd_open, pid, 0);
    if (fd < 0) return errno;
    int error = 0;
    char path[64], target[PATH_MAX + 1], *record = NULL;
    size_t size = 0;
    snprintf(path, sizeof(path), "/proc/%d/stat", pid);
    FILE *stat = fopen(path, "r");
    if (!stat || getline(&record, &size, stat) < 0) {
        error = errno ? errno : EIO;
        goto done;
    }
    char *end = strrchr(record, ')');
    if (!end || end[1] != ' ') { error = EINVAL; goto done; }
    char *save = NULL, *field = strtok_r(end + 2, " \n", &save);
    const char state = field ? field[0] : '?';
    // Field 3 is the state. Field 22 is the process start time in clock ticks.
    for (int number = 3; field && number < 22; number++) field = strtok_r(NULL, " \n", &save);
    if (!field) { error = EINVAL; goto done; }
    snprintf(path, sizeof(path), "/proc/%d/exe", pid);
    ssize_t length = readlink(path, target, PATH_MAX);
    if (length < 0 && errno != ENOENT) { error = errno; goto done; }
    if (length < 0) snprintf(target, sizeof(target), "%s", path);
    else target[length] = '\0';
    int matched = strcmp(field, birth) == 0 && strcmp(target, executable) == 0;
    int alive = matched && state != 'Z', sent = 0;
    if (alive && signum) {
        if (syscall(SYS_pidfd_send_signal, fd, signum, NULL, 0) == 0) sent = 1;
        else if (errno != ESRCH) { error = errno; goto done; }
    }
    printf("{\"matched\":%s,\"pidfdBound\":true,\"state\":\"%c\",\"alive\":%s,\"sent\":%s}\n",
           matched ? "true" : "false", state, alive ? "true" : "false", sent ? "true" : "false");
done:
    if (stat) fclose(stat);
    free(record);
    close(fd);
    return error;
}

int main(int argc, char **argv) {
    if (argc < 2 || (argc - 2) % 3) return 2;
    int signum;
    if (strcmp(argv[1], "observe") == 0) signum = 0;
    else if (strcmp(argv[1], "SIGCONT") == 0) signum = SIGCONT;
    else if (strcmp(argv[1], "SIGTERM") == 0) signum = SIGTERM;
    else if (strcmp(argv[1], "SIGKILL") == 0) signum = SIGKILL;
    else return 2;
    for (int i = 2; i < argc; i += 3) {
        char *end;
        long pid = strtol(argv[i], &end, 10);
        if (*end || pid <= 0 || pid > INT_MAX) return 2;
        int error = inspect((int)pid, argv[i + 1], argv[i + 2], signum);
        if (error == ESRCH || error == ENOENT) puts("{}");
        else if (error) printf("{\"errorNumber\":%d}\n", error);
    }
    return 0;
}
