module Main where

import Prelude

import Control.Monad.Error.Class (throwError)
import Data.Maybe (maybe)
import Effect (Effect)
import Effect.Aff (error)
import Web.DOM.Document (createTextNode)
import Web.DOM.Element as Element
import Web.DOM.Node (appendChild)
import Web.DOM.ParentNode (QuerySelector(..), querySelector)
import Web.DOM.Text as Text
import Web.HTML (window)
import Web.HTML.HTMLDocument as Document
import Web.HTML.Window as Window

main :: Effect Unit
main = do
  doc <- Window.document =<< window
  mbody <- querySelector (QuerySelector "body") (Document.toParentNode doc)
  body <- maybe (throwError (error "Could not find body")) pure mbody
  t <- createTextNode "Hello World!" (Document.toDocument doc)
  appendChild (Text.toNode t) (Element.toNode body)
