#

* faire system qui recrée les personnage: dedans faire assert pour que les controller et les entité soit cohérente avec le mode

# finir:

* graphisme:
  * [x] also maybe redo colors rgba
  * [ ] faire skybox étoile
  * [ ] faire texture pré généré ?
  * [ ] faire  que si caméra dans un mur alos l'interieur est un peu transparant: ou juste tout les z plus petit que X sont transparant !
* corriger problème un joueur gamepad tous ou alors faire qu'il faut start mais choisir !
* faire menu avec gamepad
* camera: smooth
* variables
* son

# gameplay

* on peut tirer sur les mines et roquette ? permet d'aider les moins forts, juste les roquette peut être
* des boules a prendre et des boules qui s'échappe quand on s'approche et des boules qui font des tours
* mettre le fait de tirer en option de création de map
* pour simplifier la caméra peut être faire que la vue est dans le vaisseau ? bof

* permettre de égler son niveau de tolérance explosion
  * tolérance murs
  * tolérance bombe
  * tolérance rocket ....

# graphisme

les mines et les rockette et tout ce qui bouge sont des boules avec des halo qui reste derrière pour montrer le mouvement

sauf le personnage car il vaut mieux montrer l'horizontalité

# graphics

si un joueur et présent mais pas d'entité correspondante ou celle ci est morte alors on en crée une au bout de X secondes

# Todo

* corriger bug une face n'apparait pas
* faire skybox ou juste couleur de fond: voie lactée peut être
* faire vaisseau
* faire le placement joli de la caméra (slider faust....)

# Interface

* joystick repéré le premier joueur est le premier device repéré et des qu'un device est supprimé il est de nouveau libre
* menu general:
  * number of player
  * add keyboard
  * new level

* level menu
  * TODO

* menu individuel
  * tolérance ...
  * velocity, inertia ? bof plutot pas
  * return
  * go to main menu
  * on y entre avec back et sort avec start

* on fait espace entrer pour créer un nouveau calvier: ca demande es differente tuche on valide et après c'est géré comme un gamepad (avec une touche pour disconnect)

* faire une interface basique avec rusttype

# Textures

* améliorer les texture perlin avec le crate noise

* on peut aussi faire un niveau avec que du bois
  themes:
  * bois
  * rocher avec couleurs
  * pavage comme http://dunand-chevallay-amenagement.fr/images/diapos/dalles_pierres_naturelles/Dunand_Chevallay_dalles_pierres_naturelles_2015-03__Web.jpg
    non régulier avec des texture issue de perlin ....

http://lodev.org/cgtutor/randomnoise.html
https://pdtextures.blogspot.fr/

* premier jet des textures sans localité avec juste du bruit blanc

  faire quelques textures fixe importer depuis des fichiers

* on fait un patron du cuboid
  * on fait les X generations (avec zoom et smooth)
  * on recolle en flouttant un peu les bord qui se toucherons une fois appliqué sur l'objet 3D
  * on additionne et divise

# Camera

faire que si obstacle alors avancer d'une certain manière la camera vers l'objet
et faire quelque chose de lisse, et pas directement position du player

# Idea

même graphisme que pepe mais le jeu est un spatial simulator !!!!!!!!!!!

des rockette téléguidé qui vont plus vite en ligne droite qu'en tournant

possibilité de quitter la partie navigation pour utiliser un style fps et et regarder sans tourner

on peut faire faire un effet dessin avec trait noir mais qui décroit en fonction de distance (pour éviter le problème des objets loin)
* texture avec trait noir contour.
* ou dessiner un trait englobant les faces ?

* maybe try non euclidean space (henry segerman)

* pour dessiner les labyrinthe: faire on dessine les plus grand rectangle 3D puis les + grand 2D puis les + grand 1D
  et random lorsque deux de même taille

  ou alors plus random: on choisi un bloc et on l'étend le plus possible

# level

on démarre en dehors du cube dans le lequel les boules rebondissent et tout

# camera

toujours derrière au dessus avec haut=haut du vaisseau
le mouvement peut être adouci avec un filtre passe bas ou tru du genre (comme les slider dans faust)

peut être un écran qui montre le filet ou pas et dans quel sens ?

# gameplay

* aussi faire qu'on peut tirer pour aider ses alliés a avancer

* faire que y'a toujours une vitesse comme dans rayman mais on peut aussi mettre le turbo
  est-ce que le turbo devrait aussi affecter le fait de tourner (tourner plus rapide ?)

* chercher des boules dans le grappin
  d'autre a ne pas prendre

* des monstres: grosse boules a esquiver

* roquette téléguidé a semé de bougeant beaucoup (par exemple les capacité de tourné est nulle
  idem mais inverse (comme attracted) plus lent mais capacité de tourné directe

* peut on tirer ? bof plus un jeu d'esquive 3D genre un shootemup 3D avec un vaisseau
  comment rendre une vue pour voir ce qui arrive derrière ? une vue assez éloigné du vaisseau

* comportement:
  * immobile
  * téléguidé (pathfinding)
  * attiré
  * rebondit
  * repoussé
  * téléguidé loin

  combinaison:
  * immobile/rebondit et attiré lorsqu'en vue

  faire très peu de pathfinding uniquement pour quelques monstres ?

* objectifs
  tous lorsqu'on les touchent ils détruisent le vaissue
  * prendre dans filet
  * ne pas prendre dans filet
  * transparent au filet

* mouvement:
  * inertie plus ou moins

* faire des bouncers target et des bouncer killer

* faire des untarget: des boules a ne surtout pas prendre

# couleurs monstres:

* target by net
* killer
