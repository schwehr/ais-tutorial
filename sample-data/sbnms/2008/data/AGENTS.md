# AGENTS.md — `sample-data/sbnms/2008/data/`

Twelve files `200801-5.ais.unique` … `200812-5.ais.unique` (one per month of
2008; 462–1 600 lines each, 9 085 total). Each line is one **unique message 5**
from the Boston N-AIS receivers (`r003669945`, `r003669946`, `r003669947`) in
USCG format:

```
!AIVDM,1,1,1,A,55Mq@Q02=KjmL@;?OODlE8D@U@R18DTp5DE:220lDHN<<7?UNDPSlm3kc3@H88888888880,2*5C,s26731,d-107,T54.25801979,x117264,r003669945,1199163535
```

Note `1,1,1` — fragment count 1 although message 5 is normally two fragments:
the collector stitched the payload back into a single 71-character payload
(424 bits + 2 fill bits), producing sentences longer than 82 characters. A
strict parser will reject these on length; the tutorial parser accepts them
and explains why.
