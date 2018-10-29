(name "msg")
(version "0.1.0")
(fingerprint 154dac61580ef7e88121ca67b2445f75ae9e2b9c)
(size 1 215)
(depth 5)
(typelength 1)
(lengthtag t1)
(type
  tx_pwr
  range
  (fingerprint eb647389b24d8cf28f84faa33642f60b0a66500e)
  (size 1 1)
  (depth 1)
  14
  63
  t1
  u8)
(type
  payload
  vector
  (fingerprint e845ee89dcb2e9d5facb5112daaec8a31eb50bd4)
  (size 1 201)
  (depth 2)
  u8
  200
  t1)
(type
  addr
  array
  (fingerprint ea0dda38212999bb35507220d4bcc0d8aa19efef)
  (size 10 10)
  (depth 2)
  u8
  10)
(type
  ping
  record
  (fingerprint 66e44350244ab802c03aacbceb207a68beaa2b2d)
  (size 13 213)
  (depth 3)
  (fields (field id 1 u8) (field address 2 addr) (field seq 3 u8) (field data 4 payload)))
(type
  pong
  record
  (fingerprint c70b71f4d5603a3f8019dcab3b278d05f176d801)
  (size 12 12)
  (depth 3)
  (fields (field id 1 u8) (field address 2 addr) (field seq 3 u8)))
(type
  pingpong
  union
  (fingerprint bbdf244868676c3a8a2677c6ef003c949394aa47)
  (size 13 214)
  (depth 4)
  t1
  (fields (field ping 1 ping) (field pong 2 pong)))
(type
  frame
  union
  (fingerprint dd34d559fa72fd2ad5ab4756adae61f01bf580a6)
  (size 14 215)
  (depth 5)
  t1
  (fields (field pingpong 1 pingpong)))
