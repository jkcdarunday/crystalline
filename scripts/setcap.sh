[ -f target/debug/crystalline ] \
  && sudo setcap "cap_net_admin,cap_net_raw,cap_dac_read_search,cap_sys_ptrace+pe" target/debug/crystalline \
  && echo "Set capabilities for target/debug/crystalline"

[ -f target/release/crystalline ] \
  && sudo setcap "cap_net_admin,cap_net_raw,cap_dac_read_search,cap_sys_ptrace+pe" target/release/crystalline \
  && echo "Set capabilities for target/release/crystalline"

exit 0
