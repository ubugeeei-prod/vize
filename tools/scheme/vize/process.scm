(define-module vize.process
  (use gauche.process)
  (export capture!
          run!
          run/exit-code))

(select-module vize.process)

;; Nix can expose RLIMIT_NOFILE as infinity on macOS. Gauche 0.9.15 asks
;; sysconf(_SC_OPEN_MAX) while wiring redirected child descriptors, and macOS
;; reports no finite value in that state. Limit only this process and its
;; children to the ordinary Darwin ceiling so direct-argv capture remains
;; usable; other platforms and already-finite limits are untouched.
(define (ensure-finite-descriptor-limit!)
  (receive (soft-limit hard-limit) (sys-getrlimit RLIMIT_NOFILE)
    (when (= soft-limit RLIM_INFINITY)
      (sys-setrlimit RLIMIT_NOFILE
                     (min hard-limit (- (expt 2 20) 1))))))

(ensure-finite-descriptor-limit!)

(define (command program arguments)
  (cons program arguments))

(define (process-exit-code process)
  (let1 status (process-exit-status process)
    (cond
      ((sys-wait-exited? status) (sys-wait-exit-status status))
      ((sys-wait-signaled? status)
       (errorf "command terminated by signal ~a: ~s"
               (sys-wait-termsig status)
               (process-command process)))
      (else
       (errorf "command returned an unknown wait status ~a: ~s"
               status
               (process-command process))))))

(define (run/exit-code program . arguments)
  (let1 process (run-process (command program arguments) :wait #t)
    (process-exit-code process)))

(define (run! program . arguments)
  (let1 status (apply run/exit-code program arguments)
    (unless (zero? status)
      (errorf "command exited with status ~a: ~s"
              status
              (command program arguments)))))

(define (capture! program . arguments)
  (call-with-input-process (command program arguments)
                           port->string))
