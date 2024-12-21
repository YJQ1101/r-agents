#!/usr/bin/env bash

# @describe Get the current time.

# @env LLM_OUTPUT=/dev/stdout The output path

# main() {
#     date >> "$LLM_OUTPUT"
# }

# eval "$(argc --argc-eval "$0" "$@")"
# main函数
# main() {
#     # 输出当前时间
#     date
# }

# # 如果有传参，执行传入的命令
# eval "\$1"
LC_TIME=en_US.UTF-8 date "+%A, %B %d, %Y %H:%M:%S"

