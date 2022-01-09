# Java Classfile Parser

[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE.txt)

A parser for [Java Classfiles](https://docs.oracle.com/javase/specs/jvms/se10/html/jvms-4.html), written in Rust using [nom](https://github.com/Geal/nom).

This is heavily modified from the original, for a Rust implementation of the JVM. It focuses on increasing type safety of the library, usability, and performance.
Type Safety Changes:
 - Make all indices into the Constant Pool their own type. This helps avoid accidentally using some other value, as well as helping avoid issues because they start at 1 rather than 0.
Usability:
 - Give `ConstantPoolIndexRaw` a generic parameter that indicates what the type at that index *should* be, which avoids the issues of checking at every place that it is used.
 - (Minor): Upgraded to most recent Nom
Performance:
 - Rather than parsing all the strings and attributes into a `Vec<u8>` (and Strings into *both* a `Vec<u8>` and `String`), this merely keeps around a range. This does require the file to be further accessible in the future, but it makes the code far more performant, since the majority of the time you simply ignore that data.

## Installation

Classfile Parser is available from crates.io and can be included in your Cargo enabled project like this:
 

## Implementation Status

- [x] Header
  - [x] Magic const
  - [x] Version info
- [x] Constant pool
  - [x] Constant pool size
  - [x] Constant types
    - [x] Utf8
    - [x] Integer
    - [x] Float
    - [x] Long
    - [x] Double
    - [x] Class
    - [x] String
    - [x] Fieldref
    - [x] Methodref
    - [x] InterfaceMethodref
    - [x] NameAndType
    - [x] MethodHandle
    - [x] MethodType
    - [x] InvokeDynamic
- [x] Access flags
- [x] This class
- [x] Super class
- [x] Interfaces
- [x] Fields
- [x] Methods
- [x] Attributes
  - [x] Basic attribute info block parsing
  - [ ] Known typed attributes parsing
    - [x] Critical for JVM
      - [x] ConstantValue
      - [x] Code
      - [x] StackMapTable
      - [x] Exceptions
      - [x] BootstrapMethods
    - [ ] Critical for Java SE
      - [ ] InnerClasses
      - [ ] EnclosingMethod
      - [ ] Synthetic
      - [ ] Signature
      - [ ] RuntimeVisibleAnnotations
      - [ ] RuntimeInvisibleAnnotations
      - [ ] RuntimeVisibleParameterAnnotations
      - [ ] RuntimeInvisibleParameterAnnotations
      - [ ] RuntimeVisibleTypeAnnotations
      - [ ] RuntimeInvisibleTypeAnnotations
      - [ ] AnnotationDefault
      - [ ] MethodParameters
    - [ ] Useful but not critical
      - [x] SourceFile
      - [ ] SourceDebugExtension
      - [ ] LineNumberTable
      - [ ] LocalVariableTable
      - [ ] LocalVariableTypeTable
      - [ ] Deprecated
