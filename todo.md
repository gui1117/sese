# Todo

* solve draws and load texture from files instead of creating them
* faire colonnes et vaisseau
* faire le placement joli de la caméra (slider faust....)
* faire monstres

* faire 3 vues et toile

# Barres

des barres random sont inséré:
il faut qu'elles traversent le cube, et que leur taille soit > traversé

# Interface


* joystick repéré le premier joueur est le premier device repéré et des qu'un device est supprimé il est de nouveau libre
* user story:
  game mode:
  * story
    * choix du niveau
  * arena
    choix du niveau
    preset: enumerateur qui se transforme en custom
    list des monstres avec leurs nombre
  * train alone

  > * gameplay
  >   * vitesse, inertie ???
  > soit le gameplay est choisi dans le menu soit il est choisi par niveau (et donc la story peut jouer la dessus)
  > soit il est choisi par chacun
  > soit il est pas choisi jamais..

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

* rajouter des barres comme rayman !!!!

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


# Murs

couleurs issue de pepe

differentes formes:
* bords arrondis cube+cylindre pour les bords et boules pour les angles
  ¿motif?
* avec des tige cubique ou cylindrique ou absente qui emglobent des surfaces

# optimisation

* faire que tout le monde statique soit contenu dans un vertexbuffer

# types de graphisme

* avec des dalles de différentes taille et dans un style marbre ou je sais avec perlin et couleurs de pepe
* avec des rochés placé au endroits des murs et qui forme un sorte de grotte ?
