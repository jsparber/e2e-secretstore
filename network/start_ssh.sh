#!/bin/bash

USERNAME="julian.sparber@studio.unibo.it"
HOSTS=(`cat "ercolani.txt"`)
#HOSTS=(dsfdfs marullo.cs.unibo.it morales.cs.unibo.it remendado.cs.unibo.it)

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
    PORT=30301
    SS_PORT=8011
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
      IP=`ping ${HOSTS[$i]} -c 1 -4 | cut -f 2 -d "(" | cut -f 1 -d ")" | head -n 1`
      #SecretStore node
      #ssnodes[$i]="${ss_node/0x/}@127.0.0.1:${SS_PORT}"
      ssnodes[$i]="${ss_node/0x/}@${IP}:${SS_PORT}"
      #Public node
      bootnodes[$i]="${public_node/@*/@${IP}}:${PORT}"
      #bootnodes[$i]="${public_node}"

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

    rsync -avz --delete test_network julian.sparber@studio.unibo.it@porgy.cs.unibo.it:~/
fi

rsync -avz --delete test_network julian.sparber@studio.unibo.it@porgy.cs.unibo.it:~/
NUMBER_OF_NODES=`cat test_network/size`
echo "Starting $NUMBER_OF_NODES nodes"
for (( i = 2; i <= $NUMBER_OF_NODES; i++ )) ; do
  if [ "$1" = "-c" ] ; then
    contract=$2
    ssh -f -o StrictHostKeyChecking=no -l $USERNAME ${HOSTS[$i]} "cd test_network;  sed -i '\$d' ss${i}.toml; echo 'acl_contract = \"$contract\"' >> ss${i}.toml; ~/bin/parity --config ss${i}.toml"
  else
    ssh -f -o StrictHostKeyChecking=no -l $USERNAME ${HOSTS[$i]} "cd test_network; ~/bin/parity --config ss${i}.toml"
  fi
done

cd test_network
if [ "$1" = "-c" ] ; then
  sed -i '$d' ss1.toml
  echo "acl_contract = \"$contract\"" >> ss1.toml
fi
parity --config "ss1.toml"

for (( i = 2; i <= $NUMBER_OF_NODES; i++ )) ; do
  ssh -f -o StrictHostKeyChecking=no -l $USERNAME ${HOSTS[$i]} "killall parity"
done
