# Statistic behavior contract

Normative state × input → outcome table for `src/components/statistic/statistic.vue`.
Every row is proven by `src/components/statistic/tests/behavior.test.ts`.

| #   | State                           | Input  | Outcome                                       | Proven by                                                     |
| --- | ------------------------------- | ------ | --------------------------------------------- | ------------------------------------------------------------- |
| S1  | value 1280, prefix `$`          | render | group named from the title; value `$1,280`    | `formats a prefixed value and names the group from its title` |
| S2  | value -1280, prefix `$`         | render | sign stays in front of the prefix: `-$1,280`  | `puts the minus sign before the prefix`                       |
| S3  | value 1200, compact, 1 fraction | render | `1.2K`                                        | `formats compact notation`                                    |
| S4  | omitted format props            | render | `en-US`, 0 fraction digits, standard notation | `defaults locale, fraction digits, and notation`              |
