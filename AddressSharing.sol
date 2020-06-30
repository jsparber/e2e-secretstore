pragma solidity ^0.5.0;

contract AddressSharing {
  struct User {
    address ethaddr;
    bool created;
  }

  mapping (byte32 => User) users;

  // Takes the username as a 32bit hash
  function set_user(byte32 username) public {
    require(
        users[username].created == false,
        "User already registered."
        );
    users[username].ethaddr = msg.sender;
    users[username].created = true;
  }

  // Takes the username as a 32bit hash
  function get_user(byte32 username) public view returns (address) {
    return users[username];
  }
}
