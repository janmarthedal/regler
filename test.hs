{-# OPTIONS_GHC -Wno-unrecognised-pragmas #-}
{-# HLINT ignore "Use <$>" #-}

import Parsing ( Parser, (<|>), natural, symbol, parse )

data Tree = Sum Tree Tree | Prod Tree Tree | Integer Int
            deriving Show

expr :: Parser Tree
expr = do t <- term
          do symbol "+"
             e <- expr
             return (Sum t e)
           <|> return t

term :: Parser Tree
term = do f <- factor
          do symbol "*"
             t <- term
             return (Prod f t)
           <|> return f

factor :: Parser Tree
factor = do symbol "("
            e <- expr
            symbol ")"
            return e
          <|> do n <- natural
                 return (Integer n)

eval :: String -> Tree
eval xs = case parse expr xs of
            Just (n, [])  -> n
            Just (_, out) -> error ("Unused input " ++ out)
            Nothing       -> error "Invalid input"
