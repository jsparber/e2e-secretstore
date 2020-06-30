pragma solidity ^0.5.0;

contract SSPermissions {
  struct Access {
    address owner;
    bool created;
    mapping (address => bool) allowed;
  }

  mapping (bytes32 => Access) docs;

  function allow_access(bytes32 id, address[] calldata users) external {
    require(
        msg.sender == docs[id].owner || docs[id].created == false,
        "Sender not authorized."
        );

    if (docs[id].created == false) docs[id].owner = msg.sender;
    
    for (uint i = 0; i < users.length; i++) {
      docs[id].allowed[users[i]] = true;
    }
  }

  function checkPermissions(address user, bytes32 id) public view returns (bool) {
    if (docs[id].allowed[user] == true || docs[id].owner == user) return true;
    return false;
  }
}
