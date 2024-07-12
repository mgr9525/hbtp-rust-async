
#cargo test tests::hbtp_server -- --exact

cmds="$1"

if [ -z $cmds];then
    cmds="hbtp_request"
fi

cargo test --features tokios --no-default-features -- "tests::$cmds" --exact --nocapture

echo "---------------cmds:$cmds"
