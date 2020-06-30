#!/bin/bash

USERNAME="julian.sparber@studio.unibo.it"
HOSTS=(`cat "ercolani.txt"`)
#HOSTS=(dsfdfs marullo.cs.unibo.it morales.cs.unibo.it remendado.cs.unibo.it)

export PATH="$PATH:/home/julian/Uni-Projects/MasterTesi/openethereum/target/release"
#TEMPLATE_NAME="ss${i}"
#TEMPLATE_HTTP
#TEMPLATE_SS_PORT="801${i}"
http_disabled="false"

NUMBER_OF_NODES=`cat test_network/size`
echo "Stopping $NUMBER_OF_NODES nodes"

for (( i = 2; i <= $NUMBER_OF_NODES; i++ )) ; do
  ssh -f -o StrictHostKeyChecking=no -l $USERNAME ${HOSTS[$i]} "killall parity"
done
