combustion
==========

Implements the [xboard protocol](https://www.gnu.org/software/xboard/engine-intf.html).

usage
-----

```{r, engine='bash'}
git clone git@github.com:spaceships/combustion.git
cd combustion
xboard -fcp $(realpath combustion) -fd $(realpath .) -debug
```

xboard is delightfully arcane. If you would like to see the messages exchanged,
you'll need to set `CHESSDIR`.

```{r, engine='bash'}
mkdir -p $HOME/.chess
export CHESSDIR=$HOME/.chess
tail -f $CHESSDIR/xboard.debug
```

Now run xboard as above.

license
-------

Public domain.
