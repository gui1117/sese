# Todo

* receate imgui

* arreter les lumières ..
* faire que des applats.
* avec textures regardé pour gadrillage
* faire d'abord des choses simple qu'on embellira
* peut être avec du parralax mapping

* a simple blender noise is beautiful but should also do lodev.org

* maybe a solution to remove edge artefact is to add verticial mirror per square texture and horizontal one to the texture and divide by 3
  et même faire le 2é mirroir après avoir faire laddiion et division pas 2 du premier
  et même ça plusieur fois

* créer un texture:
  * nombre de division

* faire le mouvement du vaisseau
* faire le placement joli de la caméra (slider faust....)

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

http://lodev.org/cgtutor/randomnoise.html
https://pdtextures.blogspot.fr/

* premier jet des textures sans localité avec juste du bruit blanc

  faire quelques textures fixe importer depuis des fichiers

* on fait un patron du cuboid
  * on fait les X generations (avec zoom et smooth)
  * on recolle en flouttant un peu les bord qui se toucherons une fois appliqué sur l'objet 3D
  * on additionne et divise

# Plan

* maze creation rendering
* camera rendering
* menu things

* physics and gameplay

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

  ou alors plus random: on choisi un bloc et on l'étend le plus possible <---

# level

on démarre en dehors du cube dans le lequel les boules rebondissent et tout

# camera

toujours derrière au dessus avec haut=haut du vaisseau
le mouvement peut être adouci avec un filtre passe bas ou tru du genre (comme les slider dans faust)

peut être un écran qui montre le filet ou pas et dans quel sens ?
# gameplay

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
