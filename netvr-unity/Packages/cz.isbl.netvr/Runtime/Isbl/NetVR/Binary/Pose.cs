using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Numerics;

namespace Isbl.NetVR.Binary {

    public sealed class Pose: IEquatable<Pose>, ICloneable {
        public Vec3 position;
        public Quaternion orientation;

        public Pose(Vec3 _position, Quaternion _orientation) {
            if (_position == null) throw new ArgumentNullException(nameof(_position));
            position = _position;
            if (_orientation == null) throw new ArgumentNullException(nameof(_orientation));
            orientation = _orientation;
        }

        public void Serialize(Serde.ISerializer serializer) {
            serializer.increase_container_depth();
            position.Serialize(serializer);
            orientation.Serialize(serializer);
            serializer.decrease_container_depth();
        }

        public int BincodeSerialize(byte[] outputBuffer) => BincodeSerialize(new ArraySegment<byte>(outputBuffer));

        public int BincodeSerialize(ArraySegment<byte> outputBuffer) {
            Serde.ISerializer serializer = new Bincode.BincodeSerializer(outputBuffer);
            Serialize(serializer);
            return serializer.get_buffer_offset();
        }

        public byte[] BincodeSerialize()  {
            Serde.ISerializer serializer = new Bincode.BincodeSerializer();
            Serialize(serializer);
            return serializer.get_bytes();
        }

        public static Pose Deserialize(Serde.IDeserializer deserializer) {
            deserializer.increase_container_depth();
            Pose obj = new Pose(
            	Vec3.Deserialize(deserializer),
            	Quaternion.Deserialize(deserializer));
            deserializer.decrease_container_depth();
            return obj;
        }

        public static Pose BincodeDeserialize(byte[] input) => BincodeDeserialize(new ArraySegment<byte>(input));

        public static Pose BincodeDeserialize(ArraySegment<byte> input) {
            if (input == null) {
                 throw new Serde.DeserializationException("Cannot deserialize null array");
            }
            Serde.IDeserializer deserializer = new Bincode.BincodeDeserializer(input);
            Pose value = Deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.Count) {
                 throw new Serde.DeserializationException("Some input bytes were not read");
            }
            return value;
        }
        public override bool Equals(object obj) => obj is Pose other && Equals(other);

        public static bool operator ==(Pose left, Pose right) => Equals(left, right);

        public static bool operator !=(Pose left, Pose right) => !Equals(left, right);

        public bool Equals(Pose other) {
            if (other == null) return false;
            if (ReferenceEquals(this, other)) return true;
            if (!position.Equals(other.position)) return false;
            if (!orientation.Equals(other.orientation)) return false;
            return true;
        }

        public override int GetHashCode() {
            unchecked {
                int value = 7;
                value = 31 * value + position.GetHashCode();
                value = 31 * value + orientation.GetHashCode();
                return value;
            }
        }

        /// <summary>Creates a shallow clone of the object.</summary>
        public Pose Clone() => (Pose)MemberwiseClone();

        object ICloneable.Clone() => Clone();

    }

} // end of namespace Isbl.NetVR.Binary
