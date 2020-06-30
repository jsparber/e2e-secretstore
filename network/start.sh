#!/bin/bash


export PATH="$PATH:/home/julian/Uni-Projects/MasterTesi/openethereum/target/release"
#TEMPLATE_NAME="ss${i}"
#TEMPLATE_HTTP
#TEMPLATE_SS_PORT="801${i}"
http_disabled="false"

function finish {
  sleep 0
}

trap finish SIGINT

if [ "$1" = "-s" ] ; then
  rm -rf test_network
  mkdir test_network
  cd test_network
  NUMBER_OF_NODES=$2
  echo "$NUMBER_OF_NODES" > size
  echo "Setup $NUMBER_OF_NODES nodes"
  for (( i = 1; i <= $NUMBER_OF_NODES; i++ ))
  do
    PORT=$(( 30300 + $i ))
    SS_PORT=$(( 8010 + $i ))
    sed -e "s/TEMPLATE_NAME/ss${i}/g" -e "s/TEMPLATE_PORT/${PORT}/g" -e "s/TEMPLATE_HTTP/${http_disabled}/g" -e "s/TEMPLATE_SS_PORT/${SS_PORT}/g" "../ss-template.toml" > "ss${i}.toml"

    PASSWORD="ss${i}pwd"
    echo $PASSWORD > "ss${i}.pwd"
    addr=`parity --config "ss${i}.toml" account new`

    echo "Create account with address \"${addr}\""
    sed -i -e "s/#TEMPLATE_ACCOUNT/self_secret = \"${addr/0x/}\"/g" "ss${i}.toml"

    exec 2>&1

    output=`( parity --config "ss${i}.toml" 2>&1 >/dev/null ) | \
      while read line ; do
        echo "$line" | grep "Starting SecretStore node"
        echo "$line" | grep "Public"
        if [ $? = 0 ]
        then
          #PID=$!
          # -n selects only the newest parity instance, so others shouldn't be effected
          # just be carefull not to run this script twice at the same time
          kill -INT $(pgrep -n parity)
        fi
      done`

      ss_node=`echo $output | grep -o "0x\S*"`
      public_node=`echo $output | grep -o "enode.*"`
      #SecretStore node
      ssnodes[$i]="${ss_node/0x/}@127.0.0.1:${SS_PORT}"
      #Public node
      bootnodes[$i]="${public_node}"

      echo "Created node ss${i}"
      http_disabled="true"
    done

    echo "Finished creating nodes."
    echo "SecretStore nodes:"
    for (( i = 1; i <= $NUMBER_OF_NODES; i++ )) ; do
      echo "  ${ssnodes[$i]}"
    done
    echo "Bootnodes:"
    for (( i = 1; i <= $NUMBER_OF_NODES; i++ )) ; do
      echo "  ${bootnodes[$i]}"
    done

    echo "Update bootnodes and SecretStore nodes."
    for (( i = 1; i <= $NUMBER_OF_NODES; i++ ))
    do
      string_bootnodes=""
      string_ssnodes=""
      for (( j = 1; j <= $NUMBER_OF_NODES; j++ ))
      do
        string_bootnodes+="  \"${bootnodes[$j]}\",\n"
        if [ $i -ne $j ] ; then
          string_ssnodes+="  \"${ssnodes[$j]}\",\n"
        fi
      done
      sed -i "/\[network\]/a bootnodes = \[\n${string_bootnodes}\]" "ss${i}.toml"
      sed -i "/\[secretstore\]/a nodes = \[\n${string_ssnodes}\]" "ss${i}.toml"
    done
    cd ..
fi

cd test_network
NUMBER_OF_NODES=`cat size`
echo "Starting $NUMBER_OF_NODES nodes"
for (( i = 1; i <= $NUMBER_OF_NODES; i++ )) ; do
  parity --config "ss${i}.toml" &
done

#wait that they are killed
while true; do
  wait $(pgrep -n parity -P $$) && break
done
echo "Stopped all nodes."
