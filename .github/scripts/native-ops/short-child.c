
#define _GNU_SOURCE
#include <errno.h>
#include <pthread.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>
#include <sys/syscall.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>
static void recovered(int signal_number) {
  const char marker[] = "CONTROL_HANDLED_SEGV\n";
  (void) signal_number;
  write(STDOUT_FILENO, marker, sizeof(marker) - 1);
}
static void handler(int signal_number) {
  const char marker[] = "CONTROL_SIGSEGV_HANDLER\n";
  (void) signal_number;
  write(STDOUT_FILENO, marker, sizeof(marker) - 1);
  raise(SIGABRT);
  _exit(93);
}
static void *worker(void *unused) {
  (void) unused;
  pthread_setname_np(pthread_self(), "short-fault");
  dprintf(STDOUT_FILENO, "CONTROL_THREAD %ld %ld\n",
          (long) getpid(), (long) syscall(SYS_gettid));
  struct timespec delay = { .tv_sec = 0, .tv_nsec = 5000000 };
  nanosleep(&delay, NULL);
  raise(SIGSEGV);
  _exit(94);
}
static long long ns(struct timespec value) {
  return (long long) value.tv_sec * 1000000000LL + value.tv_nsec;
}
int main(int argc, char **argv) {
  struct rlimit core_limit = {0, 0};
  if (setrlimit(RLIMIT_CORE, &core_limit) != 0) return 89;
  if (argc == 2 && strcmp(argv[1], "root-signal") == 0) {
    struct sigaction action = {0};
    action.sa_handler = recovered;
    sigemptyset(&action.sa_mask);
    sigaction(SIGSEGV, &action, NULL);
    raise(SIGSEGV);
    raise(SIGKILL);
    return 90;
  }
  if (argc == 2 && strcmp(argv[1], "parent-first") == 0) {
    pid_t intermediate = fork();
    if (intermediate < 0) return 91;
    if (intermediate == 0) {
      pid_t grandchild = fork();
      if (grandchild < 0) _exit(92);
      if (grandchild == 0) {
        execl(argv[0], argv[0], "late-worker", (char *) NULL);
        _exit(98);
      }
      _exit(17);
    }
    int status;
    while (waitpid(intermediate, &status, 0) < 0)
      if (errno != EINTR) return 99;
    return WIFEXITED(status) && WEXITSTATUS(status) == 17 ? 13 : 87;
  }
  if (argc == 2) {
    if (strcmp(argv[1], "late-worker") == 0) {
      struct timespec delay = { .tv_sec = 0, .tv_nsec = 50000000 };
      nanosleep(&delay, NULL);
    }
    struct sigaction action = {0};
    action.sa_handler = handler;
    sigemptyset(&action.sa_mask);
    sigaction(SIGSEGV, &action, NULL);
    pthread_t thread;
    if (pthread_create(&thread, NULL, worker, NULL) != 0) return 95;
    pthread_join(thread, NULL);
    return 96;
  }
  struct timespec start, end;
  clock_gettime(CLOCK_MONOTONIC, &start);
  pid_t child = fork();
  if (child < 0) return 97;
  if (child == 0) {
    execl(argv[0], argv[0], "worker", (char *) NULL);
    _exit(98);
  }
  int status;
  while (waitpid(child, &status, 0) < 0) if (errno != EINTR) return 99;
  clock_gettime(CLOCK_MONOTONIC, &end);
  int observed = WIFSIGNALED(status) ? WTERMSIG(status) : 0;
  long long elapsed = ns(end) - ns(start);
  dprintf(STDOUT_FILENO,
      "CONTROL_RESULT {\"childPid\":%ld,\"signal\":%d,\"elapsedNs\":%lld}\n",
      (long) child, observed, elapsed);
  return observed == SIGABRT && elapsed < 250000000LL ? 7 : 88;
}
